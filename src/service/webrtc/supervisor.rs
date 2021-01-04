use crate::models::webrtc;
use crate::service::webrtc::webrtc as service_webrtc;
use actix::{Actor, Addr, Context, Handler};
use log::info;
use std::collections::BTreeMap;

pub struct Supervisor {
    leader: BTreeMap<String, Addr<service_webrtc::WebRTC>>,
}

impl Actor for Supervisor {
    type Context = Context<Self>;
}

impl Supervisor {
    pub fn new() -> Addr<Supervisor> {
        let supervisor = Supervisor {
            leader: BTreeMap::new(),
        };
        supervisor.start()
    }
}

impl Handler<webrtc::RegisterUser> for Supervisor {
    type Result = ();

    fn handle(&mut self, room_leader: webrtc::RegisterUser, _: &mut Context<Self>) {
        let room_name = room_leader.room_name.clone();
        if !self.leader.contains_key(&room_name) {
            let webrtc_leader_address = service_webrtc::WebRTC::new(
                &room_leader.room_address.clone(),
                &room_leader.room_name.clone(),
                &room_leader.uuid.clone(),
            );

            self.leader
                .insert(room_leader.room_name.clone(), webrtc_leader_address);
        } else {
            info!("ADAA");
        }
    }
}

impl Handler<webrtc::SessionDescription> for Supervisor {
    type Result = ();

    fn handle(&mut self, session_description: webrtc::SessionDescription, _: &mut Context<Self>) {
        info!(
            "[GET SDP MESSAGE] [ROOM: {}] [UUID: {}], SEND TO LEADER",
            session_description.room_name, session_description.uuid
        );
        if let Some(leader_address) = self.leader.get(&session_description.room_name) {
            leader_address.do_send(session_description);
        }
    }
}

impl Handler<webrtc::ICECandidate> for Supervisor {
    type Result = ();

    fn handle(&mut self, session_description: webrtc::ICECandidate, _: &mut Context<Self>) {
        info!(
            "[GET ICE MESSAGE] [ROOM: {}] [UUID: {}], SEND TO LEADER",
            session_description.room_name, session_description.uuid
        );
        if let Some(leader_address) = self.leader.get(&session_description.room_name) {
            leader_address.do_send(session_description);
        }
    }
}

impl Handler<webrtc::DeleteLeader> for Supervisor {
    type Result = ();

    fn handle(&mut self, delete_reader: webrtc::DeleteLeader, _: &mut Context<Self>) {
        info!(
            "[DELETE LEADER] [ROOM: {}] [UUID: {}]",
            delete_reader.room_name, delete_reader.uuid
        );
        if let Some(leader_address) = self.leader.get(&delete_reader.room_name) {
            leader_address.do_send(delete_reader.clone());
        }
        self.leader.remove(&delete_reader.room_name);
    }
}
