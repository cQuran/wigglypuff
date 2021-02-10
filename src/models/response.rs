use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseBody<T> {
    Rooms(T),
    Users(T),
    IceServers(T),
    Response(T),
    Message(T),
}