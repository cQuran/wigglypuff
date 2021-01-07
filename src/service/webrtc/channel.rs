use crate::constants;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::receiver;
use actix::{Actor, Addr, Handler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gstreamer;
use gstreamer::{ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstBinExtManual};

pub struct Channel {
    receivers: BTreeMap<String, receiver::Receiver>,
    pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
}

impl Channel {
    pub fn new(room_name: &String) -> Addr<Channel> {
        let pipeline = gstreamer::Pipeline::new(Some(room_name));

        let channel = Channel {
            receivers: BTreeMap::new(),
            pipeline_gstreamer: Arc::new(Mutex::new(webrtc::GstreamerPipeline {
                pipeline: pipeline,
            })),
        };

        channel.start()
    }

    fn create_webrtc_pipeline(&self, uuid: &String, room_name: &String) -> gstreamer::Element {
        info!("MAU JALAN NEEE");
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        info!("JALAN KOKKKK");
        let previous = gstreamer::parse_launch(&format!(
            "audiotestsrc is-live=true ! opusenc ! rtpopuspay name={}_{} pt=97",
            uuid, "previous"
        ))
        .unwrap();

        let webrtcbin = gstreamer::ElementFactory::make("webrtcbin", Some(uuid))
            .expect("Could not instanciate webrtcbin");

        let next = gstreamer::parse_launch(&format!(
            "rtpopusdepay ! rtpopuspay name={}_{} pt=97",
            uuid, "next"
        )).unwrap();

        pipeline_gstreamer
            .pipeline
            .add_many(&[&previous, &webrtcbin, &next])
            .unwrap();

        let previous = pipeline_gstreamer
            .pipeline
            .get_by_name(&format!("{}_{}", uuid, "previous"))
            .expect("can't find webrtcbin");
        previous
            .link(&webrtcbin)
            .expect("element could not be linked");
        webrtcbin.set_property_from_str("stun-server", constants::STUN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        let name = room_name.clone();
        pipeline_gstreamer.pipeline.call_async(move |pipeline| {
            info!("[ROOM: {}] STARTING GSTREAMER PIPELINE", name);
            if pipeline.set_state(gstreamer::State::Playing).is_err() {
                info!("Failed to set pipeline to Playing");
            }
        });

        pipeline_gstreamer.pipeline.call_async(|pipeline| {
            pipeline
                .set_state(gstreamer::State::Playing)
                .expect("Couldn't set pipeline to Playing");
        });

        webrtcbin
    }
}

impl Actor for Channel {
    type Context = actix::Context<Self>;
}

impl Handler<supervisor::RegisterUser> for Channel {
    type Result = ();

    fn handle(&mut self, user: supervisor::RegisterUser, _: &mut actix::Context<Self>) {
        let webrtcbin = self.create_webrtc_pipeline(&user.uuid, &user.room_name);
        info!("JADIIIIIIIIIII");
        let receiver = receiver::Receiver::new(
            user.room_address,
            &user.room_name,
            &user.uuid,
            self.pipeline_gstreamer.clone(),
            webrtcbin,
        )
        .unwrap();
        self.receivers.insert(user.uuid, receiver);

        // for sender_user in self.senders {
        //     if user.uuid != sender_user.uuid {
        //         if let Some(receiver) = self.receivers.get(sender_user.uuid) {
        //             let user_receiver = webrtc::Receiver::new(
        //                 sender_user.room_address,
        //                 sender_user.room_name,
        //                 sender_user.uuid.clone()
        //             );

        //             receiver.insert(sender_user.uuid, user_receiver);
        //         }
        //         if let Some(receiver) = self.receivers.get(user.uuid) {
        //             let user_receiver = webrtc::Receiver::new(
        //                 user.room_address,
        //                 user.room_name,
        //                 user.uuid.clone()
        //             );
        //             receiver.insert(user.uuid, user_receiver);
        //         }
        //     }
        // }
    }
}

impl Handler<webrtc::SessionDescription> for Channel {
    type Result = ();

    fn handle(&mut self, sdp: webrtc::SessionDescription, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET SDP FROM CHANNEL]",
            sdp.room_name, sdp.from_uuid
        );

        if let Some(user) = self.receivers.get(&sdp.from_uuid) {
            user.on_session_answer(sdp.sdp);
        }
    }
}

impl Handler<webrtc::ICECandidate> for Channel {
    type Result = ();

    fn handle(&mut self, ice: webrtc::ICECandidate, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET ICE FROM CHANNEL]",
            ice.room_name, ice.from_uuid
        );

        if let Some(user) = self.receivers.get(&ice.from_uuid) {
            user.on_ice_answer(ice.sdp_mline_index, ice.candidate);
        }
    }
}

// impl Handler<webrtc::Kill> for Channel {
//     type Result = ();

//     fn handle(&mut self, _: webrtc::Kill, context: &mut actix::Context<Self>) {
//         context.stop();
//     }
// }

// impl Handler<webrtc::CreatePeer> for Channel {
//     type Result = ();

//     fn handle(&mut self, create_peer: webrtc::CreatePeer, context: &mut actix::Context<Self>) {
//         // peer = Peer(
//         //     create_peer.room_address,
//         //     create_peer.room_name,
//         //     create_peer.uuid,
//         //     self.leader.pipeline
//         // );
//         self.pipeline_gstreamer
//             .set_state(gstreamer::State::Null)
//             .expect("Failed to set the pipeline state to null");
//         context.stop();
//     }
// }
