use std::usize;

use axum::{
    body::{self, Body},
    response,
    routing::{get, post},
    Router,
};
use axum_macros::debug_handler;
use futures::TryStreamExt;
use hyper::Request;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::StreamReader;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/file", post(copy_body_to_another_stream))
        .route("/file_slow", post(copy_body_to_another_stream_and_use_ram));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server is running on port 3000");
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn root() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
async fn copy_body_to_another_stream(req: Request<Body>) -> response::Result<&'static str> {
    // Create a new stream (you can replace this with your own target stream)
    let mut file = File::create(format!("files/file-{}.mkv", generate_random_id()))
        .await
        .unwrap();

    // Get a mutable reference to the request body
    let body_ref = req.into_body().into_data_stream();

    let mut reader = StreamReader::new(
        body_ref.map_err(|err| tokio::io::Error::new(tokio::io::ErrorKind::Other, err)),
    );
    match tokio::io::copy(&mut reader, &mut file).await {
        Err(err) => {
            eprintln!("Error when copy - {}", err);
            drop(file);
        }
        Ok(_) => (),
    }

    Ok("Body copied to another stream successfully")
}

#[debug_handler]
async fn copy_body_to_another_stream_and_use_ram(
    req: Request<Body>,
) -> response::Result<&'static str> {
    let body_bytes = body::to_bytes(req.into_body(), usize::MAX).await.unwrap();
    // Create a new stream (you can replace this with your own target stream)
    let mut file = File::create(format!("files/file-{}.mkv", generate_random_id()))
        .await
        .unwrap();
    file.write(&body_bytes).await.unwrap();
    Ok("Body copied to another stream successfully")
}

fn generate_random_id() -> String {
    let id = uuid::Uuid::new_v4();
    return id.to_string();
}
