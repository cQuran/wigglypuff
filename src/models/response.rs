use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseBody<T> {
    Rooms(T),
    Users(T),
    Addresses(T),
    Response(T),
    Message(T),
}