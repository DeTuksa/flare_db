use std::{error::Error, sync::Arc};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

use crate::{network::message::{Message, Response}, storage::storage::Storage};

pub struct Server {
    db: Arc<Storage>
}

impl Server {
    pub fn new(path: &str) -> Self {
        Server {
            db: Arc::new(Storage::new(path))
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server is up and running on {}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let db = Arc::clone(&self.db);
            tokio::spawn(async move {
                handl_client(stream, db).await;
            });
        }
    }
}

async fn handl_client(
    mut stream: tokio::net::TcpStream,
    db: Arc<Storage>
) {
    let mut buf = [0; 1024];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => return ,
            Ok(n) => {
                let req = String::from_utf8_lossy(&buf[0..n]);
                println!("Received request: {}", req);

                let message: Result<Message, _> = serde_json::from_str(&req);
                match message {
                    Ok(msg) => {
                        let response = match msg {
                            Message::Get(key) => {
                                let value = db.get_in_memory(key.clone()).or_else(|| db.get_persistent(&key));
                                Response::Value(value)
                            }
                            Message::Set(key, value) => {
                                let success_in_mem = db.set_in_memory(key.clone(), value.clone());
                                let success_pers = db.set_persistent(key, value);
                                Response::Success(success_in_mem && success_pers)
                            }
                            Message::Delete(key) => {
                                let success_in_mem =db.delete_in_memory(key.clone());
                                let success_pers =db.delete_persistent(&key);
                                Response::Success(success_pers && success_in_mem)
                            }
                        };
                        let response_json = serde_json::to_string(&response).unwrap();
                        if let Err(e) = stream.write_all(response_json.as_bytes()).await {
                            eprintln!("Error writing response to socket: {:?}", e);
                        }
                    }
                    Err(_) => {
                        eprintln!("Unknown message format: {}", req);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream {:?}", e);
                let error_res = Response::Error("Invalid message format".to_string());
                let response_str = serde_json::to_string(&error_res).unwrap();
                if let Err(e) = stream.write_all(response_str.as_bytes()).await {
                    eprintln!("Failed to write to socket: {:?}", e);
                }
                return;
            }
        }
    }   
}