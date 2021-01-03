use crate::models::message_websocket;
use crate::service::room;
use actix::Addr;
use actix_derive::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct SessionDescription {
    pub room_name: String,
    pub uuid: String,
    #[serde(rename = "type")]
    pub types: String,
    pub sdp: String,
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CheckRunning {}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct ICECandidate {
    pub room_name: String,
    pub uuid: String,
    pub candidate: String,
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_mline_index: u32,
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct SDPAnswer {
    #[serde(rename = "type")]
    pub types: String,
    pub sdp: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateLeader {
    pub uuid: String,
    pub room_name: String,
    pub room_address: Addr<room::Room>,
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct WigglypuffWebRTC {
    pub uuid: String,
    pub room_name: String,
    pub data: message_websocket::MessageSocketType,
}

impl WigglypuffWebRTC {
    pub fn new(
        uuid: &str,
        room_name: &str,
        data: message_websocket::MessageSocketType,
    ) -> WigglypuffWebRTC {
        WigglypuffWebRTC {
            uuid: uuid.to_owned(),
            room_name: room_name.to_owned(),
            data: data.to_owned(),
        }
    }
}
