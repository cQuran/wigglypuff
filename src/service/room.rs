use crate::models::{message_websocket, room, webrtc};
use actix::{Actor, Addr, Context, Handler, Recipient};
use log::info;
use std::collections::{BTreeMap, HashSet};

pub struct Room {
    sessions: BTreeMap<String, Recipient<room::Message>>,
    rooms: BTreeMap<String, HashSet<String>>,
    masters: BTreeMap<String, String>,
}

impl Room {
    pub fn new() -> Addr<Room> {
        let room = Room {
            sessions: BTreeMap::new(),
            rooms: BTreeMap::new(),
            masters: BTreeMap::new(),
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
            .insert(connect.uuid.clone(), connect.session_address);

        self.rooms
            .entry(connect.room_name.clone())
            .or_insert_with(HashSet::new)
            .insert(connect.uuid.clone());
    }
}

impl Handler<room::KickUser> for Room {
    type Result = ();

    fn handle(&mut self, user: room::KickUser, _: &mut Context<Self>) {
        self.sessions.remove(&user.uuid);
        self.rooms
            .entry(user.room_name.clone())
            .or_insert_with(HashSet::new)
            .remove(&user.uuid);
        self.masters.remove(&user.uuid);
        let status = serde_json::to_string(&message_websocket::UserStatus {
            action: "UserLeave",
            uuid: &user.uuid,
        })
        .unwrap();

        self.broadcast(&user.uuid, &user.room_name, &status)
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
    type Result = bool;

    fn handle(&mut self, create_room: room::CreateRoom, _: &mut Context<Self>) -> Self::Result {
        match !self.masters.contains_key(&create_room.name)
            && !self.rooms.contains_key(&create_room.name)
        {
            true => {
                self.masters
                    .insert(create_room.name.clone(), create_room.master_uuid.clone());
                self.rooms
                    .insert(create_room.name.to_owned(), HashSet::new());

                true
            }
            false => false,
        }
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

impl Handler<room::GetRooms> for Room {
    type Result = <room::GetRooms as actix::Message>::Result;

    fn handle(&mut self, _: room::GetRooms, _: &mut Context<Self>) -> room::Rooms {
        let rooms: Vec<String> = self
            .rooms
            .clone()
            .into_iter()
            .map(|(room_name, _)| room_name)
            .collect();

        room::Rooms(rooms)
    }
}

impl Handler<room::GetUsers> for Room {
    type Result = <room::GetUsers as actix::Message>::Result;

    fn handle(&mut self, room: room::GetUsers, _: &mut Context<Self>) -> room::Users {
        match self.masters.get(&room.name) {
            Some(_) => {
                let users = self
                    .rooms
                    .get(&room.name)
                    .unwrap()
                    .iter()
                    .map(String::from)
                    .collect();
                return room::Users(users);
            }
            None => return room::Users(Vec::new()),
        };
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

    fn handle(&mut self, mut webrtc: webrtc::WigglypuffWebRTC, _: &mut Context<Self>) {
        match webrtc.data {
            message_websocket::MessageSocketType::WebRTCConnectionState {} => {
                let message_new_user = serde_json::to_string(&message_websocket::UserStatus {
                    action: "NewUser",
                    uuid: &webrtc.uuid,
                })
                .unwrap();
                self.broadcast(&webrtc.uuid, &webrtc.room_name, &message_new_user);
            }
            _ => {
                let (uuid_src, uuid_sink) = match webrtc.role {
                    webrtc::Role::Producer {} => (webrtc.uuid.clone(), webrtc.uuid.clone()),
                    webrtc::Role::Consumer {} => {
                        let result: Vec<&str> = webrtc.uuid.split("_sink:").collect();
                        (result[0][4..].to_string(), result[1].to_string())
                    }
                };
                webrtc.uuid = uuid_src;
                let message = serde_json::to_string(&webrtc).unwrap();
                self.send_user(&uuid_sink, &webrtc.room_name, &message);
            }
        }
    }
}

impl Room {
    fn broadcast(&self, from_uuid: &str, room_name: &str, message: &str) {
        info!(
            "[ROOM: {}] [FROM UUID: {}] [BROADCAST]",
            room_name, from_uuid
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
            "[ROOM: {}] [SEND USER] [TO UUID: {}] [MESSAGE: {}]",
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
