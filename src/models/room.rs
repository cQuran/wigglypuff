use actix::Recipient;
use actix_derive::{Message, MessageResponse};
use serde::{Deserialize, Serialize};

#[derive(Message, Deserialize)]
#[rtype(result = "bool")]
pub struct CreateRoom {
    pub name: String,
    pub master_uuid: String,
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct GetMaster {
    pub room_name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub room_name: String,
    pub uuid: String,
    pub session_address: Recipient<Message>,
}

#[derive(MessageResponse, Serialize)]
pub struct Rooms(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "Rooms")]
pub struct GetRooms {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast {
    pub room_name: String,
    pub uuid: String,
    pub message: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUser {
    pub room_name: String,
    pub uuid: String,
    pub message: String,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct KickUser {
    pub room_name: String,
    pub uuid: String,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct DeleteRoom {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Join {
    pub room_name: String,
    pub uuid: String,
}

#[derive(MessageResponse, Serialize)]
pub struct Users(pub Vec<String>);

#[derive(Message, Deserialize)]
#[rtype(result = "Users")]
pub struct GetUsers {
    pub name: String,
}
