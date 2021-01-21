use crate::constants;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::user;
use actix::{Actor, Addr, Handler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gstreamer;
use gstreamer::{ElementExt, ElementExtManual, GstBinExt, PadExtManual};

pub struct Channel {
    users: Arc<Mutex<BTreeMap<String, user::User>>>,
    peers: Arc<Mutex<BTreeMap<String, user::User>>>,
    pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
}

impl Channel {
    pub fn new(room_name: &str) -> Addr<Channel> {
        let pipeline = gstreamer::Pipeline::new(Some(room_name));
        let pipeline_gstreamer =
            Arc::new(Mutex::new(webrtc::GstreamerPipeline { pipeline: pipeline }));
        let channel = Channel {
            users: Arc::new(Mutex::new(BTreeMap::new())),
            peers: Arc::new(Mutex::new(BTreeMap::new())),
            pipeline_gstreamer: pipeline_gstreamer,
        };

        channel.start()
    }

    fn create_fakeaudio(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
        role: &webrtc::Role,
    ) -> gstreamer::Bin {
        let fakeaudio = gstreamer::parse_bin_from_description(
            &format!(
                "audiotestsrc wave=sine is-live=true ! opusenc name={uuid}_opusenc",
                uuid = uuid
            ),
            false,
        )
        .unwrap();
        if role == &webrtc::Role::Consumer {
            pipeline_gstreamer.pipeline.add(&fakeaudio).unwrap();
            fakeaudio.sync_state_with_parent().unwrap();
        }

        let opusenc = fakeaudio.get_by_name(&format!("{}_opusenc", uuid)).unwrap();

        let opusenc_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_opusenc_src", uuid)),
            &opusenc.get_static_pad("src").unwrap(),
        )
        .unwrap();

        fakeaudio.add_pad(&opusenc_pad).unwrap();

        if role == &webrtc::Role::Producer {
            pipeline_gstreamer.pipeline.add(&fakeaudio).unwrap();
        }

        fakeaudio
    }

    fn create_webrtcbin(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
        role: &webrtc::Role,
    ) -> gstreamer::Bin {
        let user = gstreamer::parse_bin_from_description(&format!(
            "rtpopuspay name={uuid}_rtpopuspay pt=97 ! webrtcbin name={uuid}_webrtcbin bundle-policy=max-bundle stun-server={stun_server} ! rtpopusdepay name={uuid}_rtpopusdepay",
            uuid = uuid,
            stun_server = constants::STUN_SERVER
        ), false).unwrap();
        if role == &webrtc::Role::Consumer {
            pipeline_gstreamer.pipeline.add(&user).unwrap();
            user.sync_state_with_parent().unwrap();
        }

        let rtpopuspay = user
            .get_by_name(&format!("{}_rtpopuspay", uuid))
            .expect("can't find rtpopuspay");

        let rtpopuspay_sink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_audiosink", uuid)),
            &rtpopuspay.get_static_pad("sink").unwrap(),
        )
        .unwrap();

        user.add_pad(&rtpopuspay_sink_pad).unwrap();

        let rtpopusdepay = user
            .get_by_name(&format!("{}_rtpopusdepay", uuid))
            .expect("can't find rtpopuspay");

        let rtpopusdepay_src_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_audiosrc", uuid)),
            &rtpopusdepay.get_static_pad("src").unwrap(),
        )
        .unwrap();
        user.add_pad(&rtpopusdepay_src_pad).unwrap();

        if role == &webrtc::Role::Producer {
            pipeline_gstreamer.pipeline.add(&user).unwrap();
        }

        user
    }

    fn create_fakesink(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
        role: &webrtc::Role,
    ) -> gstreamer::Bin {
        let fakesinkbin = gstreamer::parse_bin_from_description(
            &format!("fakesink name={uuid}_fakesink sync=false", uuid = uuid),
            false,
        )
        .unwrap();
        if role == &webrtc::Role::Consumer {
            pipeline_gstreamer.pipeline.add(&fakesinkbin).unwrap();
            fakesinkbin.sync_state_with_parent().unwrap();
        }

        let fakesink = fakesinkbin
            .get_by_name(&format!("{}_fakesink", uuid))
            .unwrap();

        let fakesink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_fakesink_sink", uuid)),
            &fakesink.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        fakesinkbin.add_pad(&fakesink_pad).unwrap();

        if role == &webrtc::Role::Producer {
            pipeline_gstreamer.pipeline.add(&fakesinkbin).unwrap();
        }

        fakesinkbin
    }

    fn create_teeadapter(
        &self,
        pipeline_gstreamer: &webrtc::GstreamerPipeline,
        uuid: &str,
        role: &webrtc::Role,
    ) -> gstreamer::Bin {
        let teebin = gstreamer::parse_bin_from_description(
            &format!("tee name={uuid}_tee", uuid = uuid),
            false,
        )
        .unwrap();
        if role == &webrtc::Role::Consumer {
            pipeline_gstreamer.pipeline.add(&teebin).unwrap();
            teebin.sync_state_with_parent().unwrap();
        }

        let tee = teebin.get_by_name(&format!("{}_tee", uuid)).unwrap();

        let tee_sink = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_sink", uuid)),
            &tee.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        teebin.add_pad(&tee_sink).unwrap();

        let tee_src = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_src", uuid)),
            &tee.get_request_pad("src_%u").unwrap(),
        )
        .unwrap();
        teebin.add_pad(&tee_src).unwrap();

        if role == &webrtc::Role::Producer {
            pipeline_gstreamer.pipeline.add(&teebin).unwrap();
        }

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

    fn build_producer(&self, uuid: &str) -> webrtc::UserPipeline {
        let role = webrtc::Role::Producer;

        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();
        let fakeaudio = self.create_fakeaudio(&pipeline_gstreamer, &uuid, &role);
        let webrtcbin = self.create_webrtcbin(&pipeline_gstreamer, &uuid, &role);
        let tee = self.create_teeadapter(&pipeline_gstreamer, &uuid, &role);
        let fakesink = self.create_fakesink(&pipeline_gstreamer, &uuid, &role);
        fakeaudio.link(&webrtcbin).unwrap();
        webrtcbin.link(&tee).unwrap();
        tee.link(&fakesink).unwrap();

        webrtc::UserPipeline {
            fakeaudio,
            webrtcbin,
            tee,
            fakesink,
            role,
        }
    }

    fn build_consumer(
        &self,
        peer_key: &str,
        uuid_src: &str,
        teebin_from_uuid_src: &gstreamer::Bin,
    ) -> webrtc::UserPipeline {
        let role = webrtc::Role::Consumer;

        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();
        let fakeaudio = self.create_fakeaudio(&pipeline_gstreamer, &peer_key, &role);
        let webrtcbin = self.create_webrtcbin(&pipeline_gstreamer, &peer_key, &role);
        let tee = self.create_teeadapter(&pipeline_gstreamer, &peer_key, &role);
        let fakesink = self.create_fakesink(&pipeline_gstreamer, &peer_key, &role);

        let tee_src = teebin_from_uuid_src
            .get_by_name(&format!("{}_tee", uuid_src))
            .unwrap();

        let audio_src_pad = tee_src.get_request_pad("src_%u").unwrap();
        let audio_block = audio_src_pad
            .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                gstreamer::PadProbeReturn::Ok
            })
            .unwrap();

        let teesrc_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_src", peer_key)),
            &audio_src_pad,
        )
        .unwrap();
        teebin_from_uuid_src.add_pad(&teesrc_pad).unwrap();

        teebin_from_uuid_src.link(&webrtcbin).unwrap();
        webrtcbin.link(&tee).unwrap();
        tee.link(&fakesink).unwrap();
        teebin_from_uuid_src.sync_state_with_parent().unwrap();
        audio_src_pad.remove_probe(audio_block);
        webrtc::UserPipeline {
            fakeaudio,
            webrtcbin,
            tee,
            fakesink,
            role,
        }
    }
}

impl Actor for Channel {
    type Context = actix::Context<Self>;
}

impl Handler<webrtc::RequestPair> for Channel {
    type Result = ();

    fn handle(&mut self, user: webrtc::RequestPair, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [TARGET: {}] [GET PAIR REQUEST FROM CHANNEL]",
            user.room_name, user.from_uuid, user.uuid
        );
        let users = self.users.lock().unwrap();

        // if let Some(user_src) = users.get(&user.from_uuid) {
        //     let peer_key = format!("src:{}_sink:{}", user.from_uuid, user.uuid);
        //     let user_pipeline =
        //         self.build_consumer(&peer_key, &user.from_uuid, &user_src.pipeline.tee);
        //     let new_peer = user::User::new(
        //         user_src.room_address.clone(),
        //         &user.room_name,
        //         &peer_key,
        //         user_pipeline,
        //     )
        //     .unwrap();
        //     let mut peers = self.peers.lock().unwrap();
        //     peers.insert(peer_key, new_peer);
        // }

        if let Some(user_src) = users.get(&user.uuid) {
            let peer_key = format!("src:{}_sink:{}", user.uuid, user.from_uuid);
            let user_pipeline = self.build_consumer(&peer_key, &user.uuid, &user_src.pipeline.tee);
            let new_peer = user::User::new(
                user_src.room_address.clone(),
                &user.room_name,
                &peer_key,
                user_pipeline,
            )
            .unwrap();
            let mut peers = self.peers.lock().unwrap();
            peers.insert(peer_key, new_peer);
        }
    }
}
impl Handler<supervisor::RegisterUser> for Channel {
    type Result = ();

    fn handle(&mut self, user: supervisor::RegisterUser, _: &mut actix::Context<Self>) {
        let mut users = self.users.lock().unwrap();

        let user_pipeline = self.build_producer(&user.uuid);
        let new_user = user::User::new(
            user.room_address.clone(),
            &user.room_name,
            &user.uuid,
            user_pipeline,
        )
        .unwrap();
        users.insert(user.uuid, new_user);

        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();
        self.play_pipeline(&pipeline_gstreamer);
    }
}

impl Handler<webrtc::SessionDescription> for Channel {
    type Result = ();

    fn handle(&mut self, sdp: webrtc::SessionDescription, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [TARGET: {}] [GET SDP FROM CHANNEL]",
            sdp.room_name, sdp.from_uuid, sdp.uuid
        );

        if sdp.from_uuid != sdp.uuid {
            if sdp.uuid.contains("_sink") {
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&sdp.uuid) {
                    peer.on_session_answer(sdp.sdp);
                }
            } else {
                let peer_key = format!("src:{}_sink:{}", sdp.uuid, sdp.from_uuid);
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&peer_key) {
                    peer.on_session_answer(sdp.sdp);
                }
            }
        } else {
            let users = self.users.lock().unwrap();
            if let Some(user) = users.get(&sdp.from_uuid) {
                user.on_session_answer(sdp.sdp);
            }
        }
    }
}

impl Handler<webrtc::ICECandidate> for Channel {
    type Result = ();

    fn handle(&mut self, ice: webrtc::ICECandidate, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [TARGET: {}] [GET ICE FROM CHANNEL]",
            ice.room_name, ice.from_uuid, ice.uuid
        );

        if ice.from_uuid != ice.uuid {
            if ice.uuid.contains("_sink") {
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&ice.uuid) {
                    peer.on_ice_answer(ice.sdp_mline_index, ice.candidate);
                }
            } else {
                let peer_key = format!("src:{}_sink:{}", ice.uuid, ice.from_uuid);
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&peer_key) {
                    peer.on_ice_answer(ice.sdp_mline_index, ice.candidate);
                }
            }
        } else {
            let users = self.users.lock().unwrap();
            if let Some(user) = users.get(&ice.from_uuid) {
                user.on_ice_answer(ice.sdp_mline_index, ice.candidate);
            }
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
    }
}
