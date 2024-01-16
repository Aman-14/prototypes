use std::{fmt::Display, sync::Arc};

use axum::{
    self,
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use deadpool_redis;
use futures::{
    sink::SinkExt,
    stream::{SplitStream, StreamExt},
};
use redis::{self, AsyncCommands};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
};

#[derive(Deserialize, Serialize, Debug)]
struct ContentMessage {
    // for simplicity to and from are usernames
    from: String,
    to: String,
    content: String,
}

impl Clone for ContentMessage {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
            content: self.content.clone(),
        }
    }
}

impl Display for ContentMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "from: {}\nto: {}\ncontent: {}\n",
            self.from, self.to, self.content
        );
    }
}

struct User {
    sc: mpsc::Sender<ContentMessage>,
    username: String,
}

struct AppState {
    /** Online users*/
    users: Mutex<Vec<User>>,
    unsend_messages: mpsc::Sender<ContentMessage>,
    redis_pool: deadpool_redis::Pool,
}

#[derive(Deserialize, Debug)]
struct UsernameParams {
    username: String,
}

impl AppState {
    async fn new(
        unsend_messsage_channel: mpsc::Sender<ContentMessage>,
        redis_pool: deadpool_redis::Pool,
    ) -> Self {
        return Self {
            users: Mutex::new(vec![]),
            unsend_messages: unsend_messsage_channel,
            redis_pool,
        };
    }

    async fn add_user(&self, username: String, sc: mpsc::Sender<ContentMessage>) {
        let mut users = self.users.lock().await;
        users.push(User { username, sc })
    }
    async fn remove_user(&self, username: String) {
        let mut users = self.users.lock().await;

        for (i, user) in users.iter().enumerate() {
            if user.username == username {
                users.remove(i);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:9999").await.unwrap();

    let (unsend_messsage_sc, unsend_message_rc) = mpsc::channel(100);

    let redis_pool = create_redis_pool().await;
    let state = Arc::new(AppState::new(unsend_messsage_sc, redis_pool).await);

    tokio::spawn(monitor_message_unsent(unsend_message_rc, state.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/indicator_count", get(get_indicator_count))
        .with_state(state);

    println!("Serving app at: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<UsernameParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("WS - {:?}", ws);
    println!("Usernmae - {:?}", params.username);
    let username = params.username.to_lowercase();
    {
        let users = state.users.lock().await;
        for user in users.iter() {
            if user.username == username {
                return (StatusCode::BAD_REQUEST, "Username already exists").into_response();
            }
        }
    }

    return ws.on_upgrade(|x| on_ws_upgrade(x, username, state));
}

async fn on_ws_upgrade(socket: WebSocket, username: String, state: Arc<AppState>) {
    let (mut sender, receiver) = socket.split();

    let (sc, mut rc) = mpsc::channel::<ContentMessage>(100);
    state.add_user(username.clone(), sc).await;

    let mut receiver_task = tokio::spawn(receive_from_client(
        username.clone(),
        receiver,
        state.clone(),
    ));

    let mut sender_task = tokio::spawn(async move {
        while let Some(message) = rc.recv().await {
            sender
                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                .await
                .unwrap();
        }
    });

    tokio::select! {
      rv_a = (&mut receiver_task) => {
            match rv_a {
                Ok(_) => println!("Receiver channel closed for - {}", username),
                Err(a) => println!("Error receiving messages {a:?}")
            }
            sender_task.abort();
        },
      rv_b = (&mut sender_task) => {
            match rv_b {
                Ok(_) => println!("Sender channel closed for - {}", username),
                Err(b) => println!("Error sending messages {b:?}")
            }
            receiver_task.abort();
        }
    };

    println!("User {username} removed");
    state.remove_user(username).await;
}

async fn receive_from_client(
    username: String,
    mut receiver: SplitStream<WebSocket>,
    state: Arc<AppState>,
) {
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(text) = message {
            match serde_json::from_str::<ContentMessage>(&text) {
                Ok(mut data) => {
                    println!("Message from - {} - {}", data.from, data.content);
                    data.from = username.clone();
                    let mut user_found = None::<bool>;
                    for user in state.users.lock().await.iter() {
                        if user.username == data.to {
                            user.sc.send(data.clone()).await.unwrap();
                            user_found = Some(true);
                            break;
                        };
                    }
                    if user_found.is_none() {
                        state.unsend_messages.send(data).await.unwrap();
                    }
                }
                Err(_) => (),
            }
            //
        } else {
            println!("Not a text message - {:?}", message);
            break;
        }
    }
}

async fn monitor_message_unsent(mut rc: mpsc::Receiver<ContentMessage>, state: Arc<AppState>) {
    while let Some(message) = rc.recv().await {
        let mut con = state.redis_pool.get().await.unwrap();
        match con.sadd::<_, _, usize>(&message.to, &message.from).await {
            Ok(res) => {
                println!("Got ok from redis - {res}")
            }
            Err(err) => println!("Failed to add member in set - {} - {}", message, err),
        };
    }
}

async fn create_redis_pool() -> deadpool_redis::Pool {
    let config = deadpool_redis::Config::from_url("redis://localhost:6379");
    let pool = config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap();
    return pool;
}

#[debug_handler]
async fn get_indicator_count(
    params: Query<UsernameParams>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut con = state.redis_pool.get().await.unwrap();
    let username = &params.username;
    let len: usize = con.scard(&username).await.unwrap();
    return len.to_string();
}
