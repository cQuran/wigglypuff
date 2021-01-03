use crate::models::webrtc;
use crate::service::webrtc_receive;
use actix::{Actor, Addr, Context, Handler};
use log::info;
use std::collections::BTreeMap;

pub struct Supervisor {
    leader: BTreeMap<String, Addr<webrtc_receive::WebRTC>>,
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

impl Handler<webrtc::CreateLeader> for Supervisor {
    type Result = ();

    fn handle(&mut self, room_leader: webrtc::CreateLeader, _: &mut Context<Self>) {
        let webrtc_leader_address = webrtc_receive::WebRTC::new(
            &room_leader.room_address.clone(),
            &room_leader.room_name.clone(),
            &room_leader.uuid.clone(),
        );

        self.leader
            .insert(room_leader.room_name.clone(), webrtc_leader_address);
    }
}

impl Handler<webrtc::SessionDescription> for Supervisor {
    type Result = ();

    fn handle(&mut self, session_description: webrtc::SessionDescription, _: &mut Context<Self>) {
        info!(
            "GET SDP MESSAGE FROM [ROOM: {}] [UUID: {}], SEND TO LEADER",
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
            "GET ICE MESSAGE FROM [ROOM: {}] [UUID: {}], SEND TO LEADER",
            session_description.room_name, session_description.uuid
        );
        if let Some(leader_address) = self.leader.get(&session_description.room_name) {
            leader_address.do_send(session_description);
        }
    }
}