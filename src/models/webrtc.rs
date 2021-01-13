use crate::models::message_websocket;
use actix_derive::Message;
use serde::{Deserialize, Serialize};

pub struct ReceiverPipeline {
    pub fakeaudio: gstreamer::Bin,
    pub webrtcbin: gstreamer::Bin,
    pub tee: gstreamer::Bin,
    pub fakesink: gstreamer::Bin,
}

#[derive(Message, Deserialize, Serialize)]
#[rtype(result = "()")]
pub struct ICECandidate {
    pub room_name: String,
    pub from_uuid: String,
    pub candidate: String,
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_mline_index: u32,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct GstreamerPipeline {
    pub pipeline: gstreamer::Pipeline,
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct SessionDescription {
    pub room_name: String,
    pub from_uuid: String,
    #[serde(rename = "type")]
    pub types: String,
    pub sdp: String,
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
