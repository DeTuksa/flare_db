use std::error::Error;

mod network;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::var("DB_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:6570".to_string());

    let server = network::server::Server::new();
    server.run(&addr).await?;
    Ok(())
}
