use std::error::Error;
use std::sync::Arc;
use std::env;

mod network;
mod storage;
mod raft;

use network::server::Server;
use raft::node::RaftNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::var("DB_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:6570".to_string());

    let node_id: u64 = env::var("NODE_ID").unwrap_or_else(|_| "1".to_string()).parse()?;
    let peers: Vec<String> = env::var("PEER_ADDRESSES")
        .unwrap_or_else(|_| "".to_string())
        .split(',')
        .map(String::from)
        .collect();

    let raft_node = Arc::new(RaftNode::new(node_id, peers));

    let server = Server::new("default");

    {
        let mut db = server.db.lock().unwrap();
        db.load_snapshot().await?;
        db.replay_log();
    }

    let server_node = raft_node.clone();
    let addr_clone = addr.clone();
    tokio::spawn(async move {
        server_node.start_server(&addr_clone).await;
    });

    let raft_node_clone = raft_node.clone();
    tokio::spawn(async move {
        raft_node_clone.run().await;
    });

    server.run(&addr).await?;
    Ok(())
}
