use crate::models::{message_websocket, room, webrtc};
use crate::service::session;

use log::info;
use serde_json;

pub fn broadcast_to_room(
    context: &mut session::Session,
    message: &message_websocket::MessageSocketType,
) {
    let message = serde_json::to_string(&message).unwrap();
    context.room_address.do_send(room::Broadcast {
        uuid: context.uuid.to_owned(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}

pub fn send_to_master(context: &mut session::Session, message: &message_websocket::MessageSocketType) {
    let message = serde_json::to_string(&message).unwrap();
    context.room_address.do_send(room::SendUser {
        uuid: context.master_uuid.clone(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}

pub fn send_to_client_webrtc(
    context: &mut session::Session,
    message: &message_websocket::MessageSocketType,
) {
    match message {
        message_websocket::MessageSocketType::ICECandidate {
            candidate,
            sdp_mline_index,
        } => {
            context.webrtc_address.do_send(webrtc::ICECandidate {
                candidate: candidate.to_owned(),
                sdp_mline_index: sdp_mline_index.to_owned(),
            });
        }
        message_websocket::MessageSocketType::SDPAnswer { types, sdp } => {
            context.webrtc_address.do_send(webrtc::SDPAnswer {
                types: types.to_owned(),
                sdp: sdp.to_owned(),
            });
        }
        _ => {
            info!("INCORRECT PATTERN");
        }
    }
}
