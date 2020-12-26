use crate::models::message::MessageSocket;
use actix_derive::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct WigglypuffWebRTC {
    pub uuid: String,
    pub room_name: String,
    pub data: MessageSocket,
}

impl WigglypuffWebRTC {
    pub fn new(uuid: &String, room_name: &String, data: MessageSocket) -> WigglypuffWebRTC {
        WigglypuffWebRTC {
            uuid: uuid.to_owned(),
            room_name: room_name.to_owned(),
            data: data.to_owned(),
        }
    }
}
