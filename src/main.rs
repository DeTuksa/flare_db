use std::error::Error;
use std::fs;
use std::path::PathBuf;

mod network;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = std::env::var("DB_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:6570".to_string());

    let path = generate_db_path("default");
    let server = network::server::Server::new(path.to_str().unwrap());
    server.run(&addr).await?;
    Ok(())
}

fn generate_db_path(project_name: &str) -> PathBuf {
    let mut base_dir = std::env::current_dir().unwrap();
    base_dir.push("databases");
    base_dir.push(project_name);
    base_dir.push("db");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&base_dir).unwrap();

    base_dir
}
