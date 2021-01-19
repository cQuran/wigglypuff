use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::channel;
use actix::{Actor, Addr, Context, Handler};
use log::info;
use std::collections::BTreeMap;

pub struct Supervisor {
    channels: BTreeMap<String, Addr<channel::Channel>>,
}

impl Actor for Supervisor {
    type Context = Context<Self>;
}

impl Supervisor {
    pub fn new() -> Addr<Supervisor> {
        let supervisor = Supervisor {
            channels: BTreeMap::new(),
        };
        supervisor.start()
    }
}

impl Handler<supervisor::RegisterUser> for Supervisor {
    type Result = ();

    fn handle(&mut self, user: supervisor::RegisterUser, _: &mut Context<Self>) {
        let room_name = user.room_name.clone();
        if !self.channels.contains_key(&room_name) {
            let channel = channel::Channel::new(&room_name.clone());

            channel.do_send(user);
            self.channels.insert(room_name.clone(), channel);
        } else {
            if let Some(channel) = self.channels.get(&user.room_name) {
                info!("CONTAINTS");
                channel.do_send(user);
            }
        }
    }
}

impl Handler<webrtc::SessionDescription> for Supervisor {
    type Result = ();

    fn handle(
        &mut self,
        session_description_request: webrtc::SessionDescription,
        _: &mut Context<Self>,
    ) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET SDP MESSAGE] [SEND TO CHANNEL]",
            session_description_request.room_name, session_description_request.from_uuid
        );
        if let Some(channel) = self.channels.get(&session_description_request.room_name) {
            channel.do_send(session_description_request);
        }
    }
}

impl Handler<webrtc::ICECandidate> for Supervisor {
    type Result = ();

    fn handle(&mut self, ice_candidate_request: webrtc::ICECandidate, _: &mut Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET ICE MESSAGE] [SEND TO CHANNEL]",
            ice_candidate_request.room_name, ice_candidate_request.from_uuid
        );
        if let Some(channel) = self.channels.get(&ice_candidate_request.room_name) {
            channel.do_send(ice_candidate_request);
        }
    }
}

impl Handler<supervisor::DeleteUser> for Supervisor {
    type Result = ();

    fn handle(&mut self, user: supervisor::DeleteUser, _: &mut Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [DELETE USER (USER)]",
            user.room_name, user.uuid
        );
        if let Some(channel) = self.channels.get(&user.room_name) {
            channel.do_send(user);
        }
    }
}

impl Handler<webrtc::CheckState> for Supervisor {
    type Result = ();

    fn handle(&mut self, check_state: webrtc::CheckState, _: &mut Context<Self>) {
        info!("[ROOM: {}]", check_state.name);
        if let Some(channel) = self.channels.get(&check_state.name) {
            channel.do_send(check_state);
        }
    }
}
