use crate::constants;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::receiver;
use actix::{Actor, Addr, Handler, StreamHandler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gstreamer;
use gstreamer::{ElementExt, ElementExtManual, GstBinExt};

pub struct Channel {
    receivers: Arc<Mutex<BTreeMap<String, receiver::Receiver>>>,
    pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
}

impl Channel {
    pub fn new(room_name: &str) -> Addr<Channel> {
        let pipeline = gstreamer::Pipeline::new(Some(room_name));
        let pipeline_gstreamer =
            Arc::new(Mutex::new(webrtc::GstreamerPipeline { pipeline: pipeline }));
        let channel = Channel {
            receivers: Arc::new(Mutex::new(BTreeMap::new())),
            pipeline_gstreamer: pipeline_gstreamer,
        };

        channel.start()
    }

    fn create_fakeaudio(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
    ) -> gstreamer::Bin {
        let fakeaudio = gstreamer::parse_bin_from_description(
            &format!(
                "audiotestsrc wave=sine is-live=true ! opusenc name={uuid}_opusenc",
                uuid = uuid
            ),
            false,
        )
        .unwrap();
        let opusenc = fakeaudio.get_by_name(&format!("{}_opusenc", uuid)).unwrap();

        let opusenc_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_opusenc_src", uuid)),
            &opusenc.get_static_pad("src").unwrap(),
        )
        .unwrap();

        fakeaudio.add_pad(&opusenc_pad).unwrap();
        pipeline_gstreamer.pipeline.add(&fakeaudio).unwrap();

        fakeaudio
    }

    fn create_webrtcbin(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
    ) -> gstreamer::Bin {
        let receiver = gstreamer::parse_bin_from_description(&format!(
            "rtpopuspay name={uuid}_rtpopuspay pt=97 ! webrtcbin name={uuid}_webrtcbin bundle-policy=max-bundle stun-server={stun_server} ! rtpopusdepay name={uuid}_rtpopusdepay",
            uuid = uuid,
            stun_server = constants::STUN_SERVER
        ), false).unwrap();

        let rtpopuspay = receiver
            .get_by_name(&format!("{}_rtpopuspay", uuid))
            .expect("can't find rtpopuspay");

        let rtpopuspay_sink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_audiosink", uuid)),
            &rtpopuspay.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        receiver.add_pad(&rtpopuspay_sink_pad).unwrap();

        let rtpopusdepay = receiver
            .get_by_name(&format!("{}_rtpopusdepay", uuid))
            .expect("can't find rtpopuspay");

        let rtpopusdepay_src_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_audiosrc", uuid)),
            &rtpopusdepay.get_static_pad("src").unwrap(),
        )
        .unwrap();
        receiver.add_pad(&rtpopusdepay_src_pad).unwrap();
        pipeline_gstreamer.pipeline.add(&receiver).unwrap();

        receiver
    }

    fn create_fakesink(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
    ) -> gstreamer::Bin {
        let fakesinkbin = gstreamer::parse_bin_from_description(
            &format!("fakesink name={uuid}_fakesink sync=false", uuid = uuid),
            false,
        )
        .unwrap();
        let fakesink = fakesinkbin
            .get_by_name(&format!("{}_fakesink", uuid))
            .unwrap();

        let fakesink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_fakesink_sink", uuid)),
            &fakesink.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        fakesinkbin.add_pad(&fakesink_pad).unwrap();
        pipeline_gstreamer.pipeline.add(&fakesinkbin).unwrap();

        fakesinkbin
    }

    fn create_teeadapter(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
    ) -> gstreamer::Bin {
        let teebin = gstreamer::parse_bin_from_description(
            &format!("tee name={uuid}_tee", uuid = uuid),
            false,
        )
        .unwrap();
        let tee = teebin.get_by_name(&format!("{}_tee", uuid)).unwrap();

        let teesink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_sink", uuid)),
            &tee.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        teebin.add_pad(&teesink_pad).unwrap();

        let teesrc_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_src", uuid)),
            &tee.get_request_pad("src_%u").unwrap(),
        )
        .unwrap();
        teebin.add_pad(&teesrc_pad).unwrap();
        pipeline_gstreamer.pipeline.add(&teebin).unwrap();

        teebin
    }

    fn play_pipeline(&self, pipeline_gstreamer: &webrtc::GstreamerPipeline) {
        pipeline_gstreamer.pipeline.call_async(move |pipeline| {
            if pipeline.set_state(gstreamer::State::Playing).is_err() {
                info!("Failed to set pipeline to Playing");
            }
        });

        pipeline_gstreamer.pipeline.call_async(|pipeline| {
            pipeline
                .set_state(gstreamer::State::Playing)
                .expect("Couldn't set pipeline to Playing");
        });
    }

    fn build_gstreamer(&self, uuid: &str) -> webrtc::ReceiverPipeline {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();
        let fakeaudio = self.create_fakeaudio(&pipeline_gstreamer, &uuid);
        let webrtcbin = self.create_webrtcbin(&pipeline_gstreamer, &uuid);
        let tee = self.create_teeadapter(&pipeline_gstreamer, &uuid);
        let fakesink = self.create_fakesink(&pipeline_gstreamer, &uuid);
        fakeaudio.link(&webrtcbin).unwrap();
        webrtcbin.link(&tee).unwrap();
        tee.link(&fakesink).unwrap();
        self.play_pipeline(&pipeline_gstreamer);

        webrtc::ReceiverPipeline {
            fakeaudio,
            webrtcbin,
            tee,
            fakesink,
        }
    }
}

impl Actor for Channel {
    type Context = actix::Context<Self>;
}

impl StreamHandler<gstreamer::Message> for Channel {
    fn handle(&mut self, _message: gstreamer::Message, _: &mut Self::Context) {
        // info!("MASUK {:#?}", message.view());
    }
}

impl Handler<supervisor::RegisterUser> for Channel {
    type Result = ();

    fn handle(&mut self, user: supervisor::RegisterUser, _: &mut actix::Context<Self>) {
        // Self::add_stream(fakeaudio.get_bus().unwrap().stream(), context);
        // Self::add_stream(receiver.get_bus().unwrap().stream(), context);
        let receiver_pipeline = self.build_gstreamer(&user.uuid);

        let new_receiver = receiver::Receiver::new(
            user.room_address,
            &user.room_name,
            &user.uuid,
            receiver_pipeline,
        )
        .unwrap();

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

        let receivers = self.receivers.lock().unwrap();
        if let Some(user) = receivers.get(&user.uuid) {
            let _ = user.stop_fakeaudio();
        }
    }
}
