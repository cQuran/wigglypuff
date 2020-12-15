use actix::{Actor, Addr, Context, Handler, Recipient};
use actix_derive::{Message, MessageResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct Room {
    sessions: HashMap<String, Recipient<Message>>,
    rooms: HashMap<String, HashSet<String>>,
    master_keys: HashMap<String, String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

impl Room {
    pub fn new() -> Addr<Room> {
        let ok = Room {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            master_keys: HashMap::new(),
        };
        ok.start()
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "String")]
pub struct Connect {
    pub address: Recipient<Message>,
}

impl Handler<Connect> for Room {
    type Result = String;

    fn handle(&mut self, message: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert("OK".to_string(), message.address);

        "OK".to_string()
    }
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CreateRoom {
    pub name: String,
    pub master_uuid: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateRoomWithKey {
    pub name: String,
    pub master_uuid: String,
    pub key: String,
}

impl Handler<CreateRoomWithKey> for Room {
    type Result = ();

    fn handle(&mut self, create_room: CreateRoomWithKey, _: &mut Context<Self>) {
        self.master_keys
            .insert(create_room.name.clone(), create_room.key.clone());
        self.rooms
            .insert(create_room.name.to_owned(), HashSet::new());
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
        self.master_keys.remove(&delete_room.name);
        self.rooms.remove(&delete_room.name);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast {
    pub room: String,
    pub message: String,
}

impl Handler<Broadcast> for Room {
    type Result = ();

    fn handle(&mut self, broadcast: Broadcast, _: &mut Context<Self>) {
        self.send(&broadcast.room, &broadcast.message);
    }
}

impl Room {
    fn send(&self, to: &str, message: &str) {
        // if let Some(sessions) = self.rooms.get(to) {
        //     for id in sessions {
        //         if let Some(address) = self.sessions.get(id) {
        //             let _ = address.do_send(Message(message.to_owned()));
        //         }
        //     }
        // }
    }
}
