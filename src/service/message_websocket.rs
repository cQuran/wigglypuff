use crate::models::webrtc;
use crate::models::{message_websocket, room};
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

pub fn send_to_master(
    context: &mut session::Session,
    message: &message_websocket::MessageSocketType,
) {
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
            uuid,
        } => {
            context.webrtc_address.do_send(webrtc::ICECandidate {
                room_name: context.room_name.clone(),
                from_uuid: context.uuid.clone(),
                uuid: uuid.to_string(),
                candidate: candidate.to_owned(),
                sdp_mline_index: sdp_mline_index.to_owned(),
            });
        }
        message_websocket::MessageSocketType::SessionDescription { types, sdp, uuid } => {
            context.webrtc_address.do_send(webrtc::SessionDescription {
                room_name: context.room_name.clone(),
                from_uuid: context.uuid.clone(),
                uuid: uuid.to_string(),
                types: types.to_owned(),
                sdp: sdp.to_owned(),
            });
        }
        message_websocket::MessageSocketType::RequestPair { uuid } => {
            context.webrtc_address.do_send(webrtc::RequestPair {
                room_name: context.room_name.clone(),
                from_uuid: context.uuid.clone(),
                uuid: uuid.to_string(),
            });
        }
        _ => {
            info!("INCORRECT PATTERN");
        }
    }
}
