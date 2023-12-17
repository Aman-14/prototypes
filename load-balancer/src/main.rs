use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("0.0.0.0:9999").await.unwrap();

    let mut manager = ServerManager::new(vec![
        "localhost:9991".to_string(),
        "localhost:9992".to_string(),
        "localhost:9993".to_string(),
        "localhost:9994".to_string(),
        "localhost:9995".to_string(),
    ]);

    println!("Waiting for connections!");

    loop {
        let (stream, _) = server.accept().await.unwrap();
        let addr = manager.get_next_server();
        tokio::spawn(async move {
            proxy_req(stream, addr).await;
        });
    }
}

struct ServerManager {
    servers: Vec<String>,
    idx: usize,
}

impl ServerManager {
    fn new(servers: Vec<String>) -> Self {
        return Self { servers, idx: 0 };
    }

    fn get_next_server(self: &mut Self) -> String {
        self.idx = self.idx % self.servers.len();
        let s = self.servers[self.idx].clone();
        self.idx += 1;
        return s;
    }
}

async fn proxy_req(mut source_stream: TcpStream, addr: String) {
    println!("Forwarding request to {}", addr);
    let res = match TcpStream::connect(&addr).await {
        Ok(mut be_stream) => io::copy_bidirectional(&mut source_stream, &mut be_stream).await,
        Err(err) => Err(err),
    };

    match res {
        Ok(..) => println!("Success from - {}", addr),
        Err(err) => eprintln!(
            "Failed to proxy request to {}. Error - {}",
            addr.to_string(),
            err
        ),
    }
}
