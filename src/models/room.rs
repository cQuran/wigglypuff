use actix::Recipient;
use actix_derive::{Message, MessageResponse};
use serde::{Deserialize, Serialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub room_name: String,
    pub uuid: String,
    pub room_address: Recipient<Message>,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CreateRoom {
    pub name: String,
    pub master_uuid: String,
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct GetMaster {
    pub room_name: String,
}

#[derive(MessageResponse, Serialize)]
pub struct ListRooms(pub Vec<String>);

#[derive(Message)]
#[rtype(result = "ListRooms")]
pub struct GetListRoom {}


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
