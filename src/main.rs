use std::error::Error;
mod network;
mod storage;

use network::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::var("DB_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:6570".to_string());

    let server = Server::new("default");

    server.db.lock().unwrap().replay_log();

    server.run(&addr).await?;
    Ok(())
}
