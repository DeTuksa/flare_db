use std::{error::Error, sync::{Arc, Mutex}};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

use crate::{network::message::{Message, Response}, storage::storage::Storage};

pub struct Server {
    pub db: Arc<Mutex<Storage>>
}

impl Server {
    pub fn new(path: &str) -> Self {
        Server {
            db: Arc::new(Mutex::new(Storage::new(path)))
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
    mut stream: tokio::net::TcpStream,
    db: Arc<Mutex<Storage>>
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
                                let db_clone = db.lock().unwrap();
                                let value = db_clone.get_in_memory(&key);
                                // .or_else(|| db_clone.get_persistent(&key));
                                Response::Value(value)
                            }
                            Message::Set(key, value) => {
                                let success_in_mem = db.lock().unwrap().set_in_memory(key.clone(), value.clone());
                                // let success_pers = db.lock().unwrap().set_persistent(key, value);
                                Response::Success(success_in_mem)
                            }
                            Message::Delete(key) => {
                                let success_in_mem =db.lock().unwrap().delete_in_memory(&key);
                                // let success_pers =db.lock().unwrap().delete_persistent(&key);
                                Response::Success(success_in_mem)
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