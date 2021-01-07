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
            "[ROOM: {}] [UUID: {}] [DELETE USER (RECEIVER)]",
            user.room_name, user.uuid
        );
        // self.pipeline
        //     .set_state(gstreamer::State::Null)
        //     .expect("Failed to set the pipeline state to null");

        // if let Some(channel) = self.users.get(&user.room_name) {
        //     channel.do_send(webrtc::Kill {});
        // }
    }
}