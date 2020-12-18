use crate::models::{
    message::MessageSocket,
    room::{Broadcast, SendUser},
    session::Session,
};

pub fn broadcast_to_room(context: &mut Session, message: &MessageSocket) {
    let message = serde_json::to_string(&message).unwrap();
    context.address.do_send(Broadcast {
        uuid: context.uuid.to_owned(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}

pub fn send_to_master(context: &mut Session, message: &MessageSocket) {
    let message = serde_json::to_string(&message).unwrap();
    context.address.do_send(SendUser {
        uuid: context.master_uuid.clone(),
        room_name: context.room_name.to_owned(),
        message: message,
    });
}