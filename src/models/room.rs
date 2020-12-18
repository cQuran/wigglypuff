use crate::models::message;
use actix::{Actor, Addr, Context, Handler, Recipient};
use actix_derive::{Message, MessageResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use log::info;

pub struct Room {
    sessions: HashMap<String, Recipient<Message>>,
    rooms: HashMap<String, HashSet<String>>,
    masters: HashMap<String, String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub room_name: String,
    pub uuid: String,
    pub address: Recipient<Message>,
}

impl Handler<Connect> for Room {
    type Result = ();

    fn handle(&mut self, connect: Connect, _: &mut Context<Self>) {
        self.sessions.insert(connect.uuid.clone(), connect.address);
        self.rooms
            .entry(connect.room_name.clone())
            .or_insert_with(HashSet::new)
            .insert(connect.uuid.clone());

        let new_user_json = serde_json::to_string(&message::UserStatus {
            action: "NewUser".to_string(),
            uuid: connect.uuid.clone(),
        })
        .unwrap();

        self.broadcast(&connect.uuid, &connect.room_name, &new_user_json)
    }
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct KickUser {
    pub room_name: String,
    pub uuid: String,
}

impl Handler<KickUser> for Room {
    type Result = ();

    fn handle(&mut self, kick_user: KickUser, _: &mut Context<Self>) {
        self.sessions.remove(&kick_user.uuid);
        self.rooms
            .entry(kick_user.room_name.clone())
            .or_insert_with(HashSet::new)
            .remove(&kick_user.uuid);
        self.masters.remove(&kick_user.uuid);
        let user_kicked_json = serde_json::to_string(&message::UserStatus {
            action: "UserLeave".to_string(),
            uuid: kick_user.uuid.clone(),
        })
        .unwrap();

        self.broadcast(&kick_user.uuid, &kick_user.room_name, &user_kicked_json)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendUser {
    pub room_name: String,
    pub uuid: String,
    pub message: String,
}

impl Handler<SendUser> for Room {
    type Result = ();

    fn handle(&mut self, user: SendUser, _: &mut Context<Self>) {
        self.send_user(&user.uuid, &user.room_name, &user.message);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast {
    pub room_name: String,
    pub uuid: String,
    pub message: String,
}

impl Handler<Broadcast> for Room {
    type Result = ();

    fn handle(&mut self, broadcast: Broadcast, _: &mut Context<Self>) {
        self.broadcast(&broadcast.uuid, &broadcast.room_name, &broadcast.message);
    }
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CreateRoom {
    pub name: String,
    pub master_uuid: String,
}

impl Handler<CreateRoom> for Room {
    type Result = ();

    fn handle(&mut self, create_room: CreateRoom, _: &mut Context<Self>) {
        self.masters
            .insert(create_room.name.clone(), create_room.master_uuid.clone());
        self.rooms
            .insert(create_room.name.to_owned(), HashSet::new());
    }
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct GetMaster {
    pub room_name: String,
}

impl Handler<GetMaster> for Room {
    type Result = String;

    fn handle(&mut self, master_request: GetMaster, _: &mut Context<Self>) -> String {
        match self.masters.get(&master_request.room_name) {
            Some(master_uuid) => return master_uuid.to_owned(),
            None => return "NAN".to_string(),
        }
    }
}

#[derive(MessageResponse, Serialize)]
pub struct ListRooms(Vec<String>);

#[derive(Message)]
#[rtype(result = "ListRooms")]
pub struct GetListRoom {}

impl Handler<GetListRoom> for Room {
    type Result = <GetListRoom as actix::Message>::Result;

    fn handle(&mut self, _: GetListRoom, _: &mut Context<Self>) -> ListRooms {
        let all_rooms: Vec<String> = self
            .rooms
            .clone()
            .into_iter()
            .map(|(room_name, _)| room_name)
            .collect();

        ListRooms(all_rooms)
    }
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct DeleteRoom {
    pub name: String,
}

impl Handler<DeleteRoom> for Room {
    type Result = <DeleteRoom as actix::Message>::Result;

    fn handle(&mut self, delete_room: DeleteRoom, _: &mut Context<Self>) {
        self.masters.remove(&delete_room.name);
        if let Some(sessions) = self.rooms.get(&delete_room.name) {
            for session in sessions {
                self.sessions.remove(session);
            }
        }
        self.rooms.remove(&delete_room.name);
        if let Some(sessions) = self.rooms.get(&delete_room.name) {
            for session in sessions {
                if let Some(address) = self.sessions.get(session) {
                    let _ = address.do_send(Message(delete_room.name.to_string()));
                }
            }
        }

        let user_status_remove = serde_json::to_string(&message::UserStatus {
            action: "room-removed".to_string(),
            uuid: "all".to_string(),
        })
        .unwrap();
        self.broadcast(
            &"wigglypuff".to_string(),
            &delete_room.name,
            &user_status_remove,
        );
    }
}

impl Room {
    fn broadcast(&self, from_uuid: &str, room_name: &str, message: &str) {
        info!("[BROADCAST] [ROOM: {}] [FROM UUID: {}] [MESSAGE: {}]", room_name, from_uuid, message);
        if let Some(sessions) = self.rooms.get(room_name) {
            for session in sessions {
                if *session != from_uuid {
                    if let Some(address) = self.sessions.get(session) {
                        let _ = address.do_send(Message(message.to_string()));
                    }
                }
            }
        }
    }
    fn send_user(&self, to_uuid: &str, room_name: &str, message: &str) {
        info!("[SEND USER] [ROOM: {}] [TO UUID: {}] [MESSAGE: {}]", room_name, to_uuid, message);
        if let Some(sessions) = self.rooms.get(room_name) {
            for session in sessions {
                if *session == to_uuid {
                    if let Some(address) = self.sessions.get(session) {
                        let _ = address.do_send(Message(message.to_string()));
                    }
                }
            }
        }
    }
}
