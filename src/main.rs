use serde::Deserialize;
use std::fs;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Deserialize)]
struct Config {
    listen_addr: String,
    backends: Vec<String>,
}

#[derive(Debug, Error)]
enum Error {
    #[error("failed to read config file: {0}")]
    ConfigIo(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
}

fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

async fn run(config: Config) -> Result<(), Error> {
    let listener = TcpListener::bind(config.listen_addr).await?;
    loop {
        let (socket, _addr) = match listener.accept().await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Accept Error {}", e);
                continue;
            }
        };

        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096];
    loop {
        let n = stream.read(&mut buffer).await.unwrap(); // Todo : Handle via better Error handling
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n]).await.unwrap(); // Todo : Handle via better Error handling
    }
}

fn main() -> Result<(), Error> {
    let config = load_config()?;

    println!("Listening on: {}", config.listen_addr);
    println!("Backend count: {}", config.backends.len());

    let _ = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(config));

    Ok(())
}
