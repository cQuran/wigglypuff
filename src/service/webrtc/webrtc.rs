use crate::models::webrtc;
use crate::service::room as service_room;
use crate::service::webrtc::leader::Leader;
use actix::{Actor, ActorContext, Addr, Handler};
use log::info;

use gstreamer;
use gstreamer::{prelude::ObjectExt, ElementExtManual};


macro_rules! upgrade_weak_reference {
    ($x:ident, $r:expr) => {{
        match $x.upgrade_to_strong_reference() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak_reference!($x, ())
    };
}

pub struct WebRTC {
    leader: Leader,
}

impl WebRTC {
    pub fn new(
        room_address: &Addr<service_room::Room>,
        room_name: &String,
        uuid: &String,
    ) -> Addr<WebRTC> {
        let leader = Leader::new(&room_address, &room_name, &uuid).unwrap();

        let webrtc = WebRTC { leader: leader };
        webrtc.start()
    }
}

impl Actor for WebRTC {
    type Context = actix::Context<Self>;
}

impl Handler<webrtc::SessionDescription> for WebRTC {
    type Result = ();

    fn handle(&mut self, sdp: webrtc::SessionDescription, _: &mut actix::Context<Self>) {
        let request_sdp = serde_json::to_string(&sdp).unwrap();
        info!(
            "[GET SDP FROM SUPERVISOR] [ROOM: {}] [UUID: {}] {}",
            self.leader.room_name, self.leader.uuid, request_sdp
        );
        let ret = gstreamer_sdp::SDPMessage::parse_buffer(sdp.sdp.as_bytes())
            .map_err(|_| info!("Failed to parse SDP offer"))
            .unwrap();
        let app_clone = self.leader.downgrade_to_weak_reference();

        self.leader.pipeline.call_async(move |_pipeline| {
            let app = upgrade_weak_reference!(app_clone);

            let answer = gstreamer_webrtc::WebRTCSessionDescription::new(
                gstreamer_webrtc::WebRTCSDPType::Answer,
                ret,
            );
            app.webrtcbin
                .emit(
                    "set-remote-description",
                    &[&answer, &None::<gstreamer::Promise>],
                )
                .unwrap();
        });
    }
}

impl Handler<webrtc::ICECandidate> for WebRTC {
    type Result = ();

    fn handle(&mut self, channel: webrtc::ICECandidate, _: &mut actix::Context<Self>) {
        self.leader
            .webrtcbin
            .emit(
                "add-ice-candidate",
                &[&channel.sdp_mline_index, &channel.candidate],
            )
            .unwrap();
    }
}

impl Handler<webrtc::DeleteLeader> for WebRTC {
    type Result = ();

    fn handle(&mut self, _: webrtc::DeleteLeader, context: &mut actix::Context<Self>) {
        self.leader
            .pipeline
            .set_state(gstreamer::State::Null)
            .expect("Failed to set the pipeline state to null");
        context.stop();
    }
}