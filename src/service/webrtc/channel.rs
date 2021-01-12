use crate::constants;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::receiver;
use actix::{Actor, Addr, Handler, StreamHandler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gstreamer;
use gstreamer::{
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstBinExtManual, PadExtManual,
};

pub struct Channel {
    receivers: Arc<Mutex<BTreeMap<String, receiver::Receiver>>>,
    pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
    peer_audiomixer: Arc<Mutex<BTreeMap<String, (gstreamer::Pad, gstreamer::Element)>>>,
}

impl Channel {
    pub fn new(room_name: &String) -> Addr<Channel> {
        let pipeline = gstreamer::Pipeline::new(Some(room_name));
        let channel = Channel {
            receivers: Arc::new(Mutex::new(BTreeMap::new())),
            pipeline_gstreamer: Arc::new(Mutex::new(webrtc::GstreamerPipeline {
                pipeline: pipeline,
            })),
            peer_audiomixer: Arc::new(Mutex::new(BTreeMap::new())),
        };

        channel.start()
    }

    fn create_sample_peer(&self, pipeline_gstreamer: &webrtc::GstreamerPipeline, uuid: &String) {
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
        audiomixer.link(&opusenc).unwrap();
        opusenc.link(&rtpopuspay).unwrap();
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

impl StreamHandler<gstreamer::Message> for Channel {
    fn handle(&mut self, message: gstreamer::Message, _: &mut Self::Context) {
        // info!("MASUK {:#?}", message);
    }
}

impl Handler<supervisor::RegisterUser> for Channel {
    type Result = ();

    fn handle(&mut self, user: supervisor::RegisterUser, context: &mut actix::Context<Self>) {
        let webrtcbin = self.create_webrtc_pipeline(&user.uuid, &user.room_name);

        Self::add_stream(webrtcbin.get_bus().unwrap().stream(), context);

        let new_receiver = receiver::Receiver::new(
            user.room_address,
            &user.room_name,
            &user.uuid,
            self.pipeline_gstreamer.clone(),
            webrtcbin,
            self.peer_audiomixer.clone(),
        )
        .unwrap();
        info!("REGISTER USER");
        let mut receivers = self.receivers.lock().unwrap();
        receivers.insert(user.uuid, new_receiver);
    }
}

impl Handler<webrtc::SessionDescription> for Channel {
    type Result = ();

    fn handle(&mut self, sdp: webrtc::SessionDescription, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [GET SDP FROM CHANNEL]",
            sdp.room_name, sdp.from_uuid
        );

        let receivers = self.receivers.lock().unwrap();
        if let Some(user) = receivers.get(&sdp.from_uuid) {
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

        let receivers = self.receivers.lock().unwrap();
        if let Some(user) = receivers.get(&ice.from_uuid) {
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
        // let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        // pipeline_gstreamer.pipeline.call_async(move |pipeline| {
            // let decodebin = pipeline
            //     .get_by_name(&format!("{}_decodebin", user.uuid))
            //     .expect("can't find webrtcbin");

            // let audiotestsrc = pipeline
            //     .get_by_name(&format!("{}_audiotestsrc", user.uuid))
            //     .expect("can't find webrtcbin");

            // let audiotestsrc_pad = audiotestsrc.get_static_pad("src").unwrap();

            // let audio_block = audiotestsrc_pad
            //     .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
            //         gstreamer::PadProbeReturn::Ok
            //     })
            //     .unwrap();

            // audiotestsrc
            //     .set_state(gstreamer::State::Null)
            //     .expect("Couldn't set pipeline to Playing");
            // audiotestsrc_pad.remove_probe(audio_block);
            // pipeline.remove(&audiotestsrc);
            // pipeline
            //     .set_state(gstreamer::State::Playing)
            //     .expect("Couldn't set pipeline to Playing");
            // info!("STOPPP");
        // });

        // let autoaudiosink_to_uuid = pipeline_gstreamer
        //     .pipeline
        //     .get_by_name(&format!("{}_autoaudiosink", user.to_uuid))
        //     .expect("can't find webrtcbin");

        // audioresample.unlink(&autoaudiosink_from_uuid);
        // audioresample.link(&autoaudiosink_to_uuid).unwrap();
    }
}
