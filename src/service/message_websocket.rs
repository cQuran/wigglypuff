use crate::models::{
    message::MessageSocket,
    room::{Broadcast, SendUser},
    session::Session,
    webrtc,
};
use log::info;

pub fn broadcast_to_room(context: &mut Session, message: &MessageSocket) {
    let message = serde_json::to_string(&message).unwrap();
    context.room_address.do_send(Broadcast {
        uuid: context.uuid.to_owned(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}

pub fn send_to_master(context: &mut Session, message: &MessageSocket) {
    let message = serde_json::to_string(&message).unwrap();
    context.room_address.do_send(SendUser {
        uuid: context.master_uuid.clone(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}

pub fn send_to_client_webrtc(context: &mut Session, message: &MessageSocket) {
    match message {
        MessageSocket::ICECandidate {
            candidate,
            sdp_mline_index,
        } => {
            context.webrtc_address.do_send(webrtc::ICECandidate {
                candidate: candidate.to_owned(),
                sdp_mline_index: sdp_mline_index.to_owned(),
            });
        }
        _ => {
            info!("INCORRECT PATTERN");
        }
    }
}
