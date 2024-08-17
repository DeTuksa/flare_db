use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Get(String),
    Set(String, String),
    Delete(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Value(Option<String>),
    Success(bool),
    Error(String)
}