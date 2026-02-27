use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub async fn run(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", listener.local_addr()?);

    loop {
        match listener.accept().await {
            Ok((socket, peer_addr)) => {
                tokio::spawn(handle_connection(socket, peer_addr));
            }
            Err(e) => {
                println!("Accept error: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(socket: TcpStream, peer_addr: SocketAddr) {
    println!("New connection from {}", peer_addr);
    drop(socket);
}
