use crate::constants;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::receiver;
use actix::{Actor, Addr, Handler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gstreamer;
use gstreamer::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstBinExtManual, PadExtManual,
};

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

    fn create_sample_peer(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &String
    ) {
        let audiotestsrc = gstreamer::parse_launch(&format!(
            "audiotestsrc name={uuid}_audiotestsrc wave=silence is-live=true",
            uuid = uuid
        ))
        .unwrap();
        pipeline_gstreamer
            .pipeline
            .add_many(&[&audiotestsrc])
            .unwrap();

        let audiomixer = pipeline_gstreamer
            .pipeline
            .get_by_name(&format!("{}_audiomixer", uuid))
            .expect("can't find webrtcbin");

        let audiotestsrc_pad = audiotestsrc.get_static_pad("src").unwrap();
        let audiomixer_pad = audiomixer.get_request_pad("sink_%u").unwrap();
        audiotestsrc_pad.link(&audiomixer_pad).unwrap();
    }

    fn create_webrtc_pipeline(&self, uuid: &String, room_name: &String) -> gstreamer::Element {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        let audiomixer =
            gstreamer::ElementFactory::make("audiomixer", Some(&format!("{}_audiomixer", uuid)))
                .unwrap();

        let opusenc =
            gstreamer::ElementFactory::make("opusenc", Some(&format!("{}_opusenc", uuid))).unwrap();

        let rtpopuspay =
            gstreamer::ElementFactory::make("rtpopuspay", Some(&format!("{}_rtpopuspay", uuid)))
                .unwrap();

        let webrtcbin = gstreamer::ElementFactory::make("webrtcbin", Some(uuid))
            .expect("Could not instanciate webrtcbin");

        webrtcbin.set_property_from_str("stun-server", constants::STUN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        pipeline_gstreamer
            .pipeline
            .add_many(&[&audiomixer, &opusenc, &rtpopuspay, &webrtcbin])
            .unwrap();
        rtpopuspay.set_property_from_str("pt", "97");

        self.create_sample_peer(&pipeline_gstreamer, &uuid);
        opusenc.link(&rtpopuspay).unwrap();
        audiomixer.link(&opusenc).unwrap();
        rtpopuspay.link(&webrtcbin).unwrap();

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
        info!("LEN {}", self.receivers.len());
        let webrtcbin = self.create_webrtc_pipeline(&user.uuid, &user.room_name);
        let receiver = receiver::Receiver::new(
            user.room_address,
            &user.room_name,
            &user.uuid,
            self.pipeline_gstreamer.clone(),
            webrtcbin,
        )
        .unwrap();
        self.receivers.insert(user.uuid, receiver);
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

impl Handler<supervisor::DeleteUser> for Channel {
    type Result = ();

    fn handle(&mut self, user: supervisor::DeleteUser, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET user FROM CHANNEL TEST]",
            user.room_name, user.uuid
        );
        // let audioresample = pipeline_gstreamer
        //     .pipeline
        //     .get_by_name(&format!("{}_audioresample", user.uuid))
        //     .expect("can't find webrtcbin");

        // let autoaudiosink_from_uuid = pipeline_gstreamer
        //     .pipeline
        //     .get_by_name(&format!("{}_autoaudiosink", user.uuid))
        //     .expect("can't find webrtcbin");

        // let autoaudiosink_to_uuid = pipeline_gstreamer
        //     .pipeline
        //     .get_by_name(&format!("{}_autoaudiosink", user.to_uuid))
        //     .expect("can't find webrtcbin");

        // audioresample.unlink(&autoaudiosink_from_uuid);
        // audioresample.link(&autoaudiosink_to_uuid).unwrap();
    }
}
