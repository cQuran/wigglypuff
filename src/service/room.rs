use crate::models::{message_websocket, room, webrtc};
use actix::{Actor, Addr, Context, Handler, Recipient};
use log::info;
use std::collections::{HashMap, HashSet};

pub struct Room {
    sessions: HashMap<String, Recipient<room::Message>>,
    rooms: HashMap<String, HashSet<String>>,
    masters: HashMap<String, String>,
}

impl Room {
    pub fn new() -> Addr<Room> {
        let room = Room {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            masters: HashMap::new(),
        };
        room.start()
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Handler<room::Connect> for Room {
    type Result = ();

    fn handle(&mut self, connect: room::Connect, _: &mut Context<Self>) {
        self.sessions
            .insert(connect.uuid.clone(), connect.room_address);

        self.rooms
            .entry(connect.room_name.clone())
            .or_insert_with(HashSet::new)
            .insert(connect.uuid.clone());

        let message_new_user = serde_json::to_string(&message_websocket::UserStatus {
            action: "NewUser",
            uuid: &connect.uuid,
        })
        .unwrap();

        self.broadcast(&connect.uuid, &connect.room_name, &message_new_user)
    }
}

impl Handler<room::KickUser> for Room {
    type Result = ();

    fn handle(&mut self, kick_user: room::KickUser, _: &mut Context<Self>) {
        self.sessions.remove(&kick_user.uuid);
        self.rooms
            .entry(kick_user.room_name.clone())
            .or_insert_with(HashSet::new)
            .remove(&kick_user.uuid);
        self.masters.remove(&kick_user.uuid);
        let message_kick_user = serde_json::to_string(&message_websocket::UserStatus {
            action: "UserLeave",
            uuid: &kick_user.uuid,
        })
        .unwrap();

        self.broadcast(&kick_user.uuid, &kick_user.room_name, &message_kick_user)
    }
}

impl Handler<room::SendUser> for Room {
    type Result = ();

    fn handle(&mut self, user: room::SendUser, _: &mut Context<Self>) {
        self.send_user(&user.uuid, &user.room_name, &user.message);
    }
}

impl Handler<room::Broadcast> for Room {
    type Result = ();

    fn handle(&mut self, broadcast: room::Broadcast, _: &mut Context<Self>) {
        self.broadcast(&broadcast.uuid, &broadcast.room_name, &broadcast.message);
    }
}

impl Handler<room::CreateRoom> for Room {
    type Result = ();

    fn handle(&mut self, create_room: room::CreateRoom, _: &mut Context<Self>) {
        self.masters
            .insert(create_room.name.clone(), create_room.master_uuid.clone());
        self.rooms
            .insert(create_room.name.to_owned(), HashSet::new());
    }
}

impl Handler<room::GetMaster> for Room {
    type Result = String;

    fn handle(&mut self, master_request: room::GetMaster, _: &mut Context<Self>) -> String {
        match self.masters.get(&master_request.room_name) {
            Some(master_uuid) => return master_uuid.to_owned(),
            None => return "NAN".to_string(),
        }
    }
}

impl Handler<room::GetListRoom> for Room {
    type Result = <room::GetListRoom as actix::Message>::Result;

    fn handle(&mut self, _: room::GetListRoom, _: &mut Context<Self>) -> room::ListRooms {
        let all_rooms: Vec<String> = self
            .rooms
            .clone()
            .into_iter()
            .map(|(room_name, _)| room_name)
            .collect();

        room::ListRooms(all_rooms)
    }
}

impl Handler<room::DeleteRoom> for Room {
    type Result = <room::DeleteRoom as actix::Message>::Result;

    fn handle(&mut self, delete_room: room::DeleteRoom, _: &mut Context<Self>) {
        self.masters.remove(&delete_room.name);
        if let Some(sessions) = self.rooms.get(&delete_room.name) {
            for session in sessions {
                self.sessions.remove(session);
            }
        }
        self.rooms.remove(&delete_room.name);
        if let Some(sessions) = self.rooms.get(&delete_room.name) {
            for session in sessions {
                if let Some(room_address) = self.sessions.get(session) {
                    let _ = room_address.do_send(room::Message(delete_room.name.to_string()));
                }
            }
        }

        let message_remove_user = serde_json::to_string(&message_websocket::UserStatus {
            action: "room-removed",
            uuid: "all",
        })
        .unwrap();

        self.broadcast(
            &"wigglypuff".to_string(),
            &delete_room.name,
            &message_remove_user,
        );
    }
}

impl Handler<webrtc::WigglypuffWebRTC> for Room {
    type Result = ();

    fn handle(&mut self, webrtc: webrtc::WigglypuffWebRTC, _: &mut Context<Self>) {
        let message = serde_json::to_string(&webrtc).unwrap();
        self.send_user(&webrtc.uuid, &webrtc.room_name, &message);
    }
}

impl Room {
    fn broadcast(&self, from_uuid: &str, room_name: &str, message: &str) {
        info!(
            "[BROADCAST] [ROOM: {}] [FROM UUID: {}] [MESSAGE: {}]",
            room_name, from_uuid, message
        );
        if let Some(sessions) = self.rooms.get(room_name) {
            for session in sessions {
                if *session != from_uuid {
                    if let Some(room_address) = self.sessions.get(session) {
                        let _ = room_address.do_send(room::Message(message.to_string()));
                    }
                }
            }
        }
    }
    fn send_user(&self, to_uuid: &str, room_name: &str, message: &str) {
        info!(
            "[SEND USER] [ROOM: {}] [TO UUID: {}] [MESSAGE: {}]",
            room_name, to_uuid, message
        );
        if let Some(sessions) = self.rooms.get(room_name) {
            for session in sessions {
                if *session == to_uuid {
                    if let Some(room_address) = self.sessions.get(session) {
                        let _ = room_address.do_send(room::Message(message.to_string()));
                    }
                }
            }
        }
    }
}
