use std::error::Error;

use tokio::net::TcpListener;

use crate::storage::in_memory_db::InMemoryDB;

pub struct Server {
    db: InMemoryDB
}

impl Server {
    pub fn new() -> Self {
        Server {
            db: InMemoryDB::new()
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server is up and running on {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let db = self.db.clone();
            tokio::spawn(async move {
                handl_client(stream, db).await;
            });
        }
    }
}

async fn handl_client(
    stream: tokio::net::TcpStream,
    db: InMemoryDB
) {
    
}