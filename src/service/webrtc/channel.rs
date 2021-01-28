use crate::constants;
use crate::models::message_websocket;
use crate::models::room;
use crate::models::supervisor;
use crate::models::webrtc;
use crate::service::webrtc::user;
use actix::{Actor, Addr, Arbiter, AsyncContext, Handler};
use log::info;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use glib::ObjectExt;
use gstreamer;
use gstreamer::{ElementExt, ElementExtManual, GstBinExt, PadExt, PadExtManual};

pub struct Channel {
    users: Arc<Mutex<BTreeMap<String, user::User>>>,
    peers: Mutex<BTreeMap<String, user::User>>,
    pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
}

impl Channel {
    pub fn new(room_name: &str) -> Addr<Channel> {
        let pipeline = gstreamer::Pipeline::new(Some(room_name));
        let pipeline_gstreamer =
            Arc::new(Mutex::new(webrtc::GstreamerPipeline { pipeline: pipeline }));
        let channel = Channel {
            users: Arc::new(Mutex::new(BTreeMap::new())),
            peers: Mutex::new(BTreeMap::new()),
            pipeline_gstreamer: pipeline_gstreamer,
        };
        channel.start()
    }

    async fn heartbeat(
        arc_users: Arc<Mutex<BTreeMap<String, user::User>>>,
        pipeline_gstreamer: Arc<Mutex<webrtc::GstreamerPipeline>>,
    ) {
        for (uuid, user) in arc_users.lock().unwrap().iter() {
            let pipeline = pipeline_gstreamer.lock().unwrap();

            let uuid_clone = uuid.clone();
            let room_name_clone = user.room_name.clone();
            let room_address_clone = user.room_address.clone();
            let promise = gstreamer::Promise::with_change_func(move |reply| {
                if reply.unwrap().unwrap().n_fields() < 7 {
                    let user_disconnected_json_message =
                        serde_json::to_string(&message_websocket::UserStatus {
                            action: "UserLeave",
                            uuid: &uuid_clone,
                        })
                        .unwrap();

                    info!("[OFFLINE] [{}]", uuid_clone);
                    room_address_clone.do_send(room::Broadcast {
                        uuid: uuid_clone,
                        room_name: room_name_clone,
                        message: user_disconnected_json_message,
                    });
                }
            });
            let webrtcbin = pipeline
                .pipeline
                .get_by_name(&format!("{}_webrtcbin", uuid))
                .unwrap();
            webrtcbin
                .emit("get-stats", &[&None::<gstreamer::Pad>, &promise])
                .unwrap();
        }
    }

    fn create_fakeaudio(&self, uuid: &str) -> gstreamer::Bin {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

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
        opusenc_pad.set_active(true).unwrap();
        fakeaudio.add_pad(&opusenc_pad).unwrap();

        pipeline_gstreamer.pipeline.add(&fakeaudio).unwrap();
        fakeaudio
    }

    fn create_webrtcbin(&self, uuid: &str) -> gstreamer::Bin {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        let user = gstreamer::parse_bin_from_description(&format!(
            "rtpopuspay name={uuid}_rtpopuspay pt=97 ! webrtcbin name={uuid}_webrtcbin bundle-policy=max-bundle stun-server={stun_server} turn-server={turn_server}",
            uuid = uuid,
            stun_server = constants::STUN_SERVER,
            turn_server = constants::TURN_SERVER,
        ), false).unwrap();

        let rtpopuspay = user
            .get_by_name(&format!("{}_rtpopuspay", uuid))
            .expect("can't find rtpopuspay");

        let rtpopuspay_sink_pad = gstreamer::GhostPad::with_target(
            Some(&format!("{}_audiosink", uuid)),
            &rtpopuspay.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        rtpopuspay_sink_pad.set_active(true).unwrap();
        user.add_pad(&rtpopuspay_sink_pad).unwrap();

        pipeline_gstreamer.pipeline.add(&user).unwrap();

        user
    }

    fn create_fakesink(&self, uuid: &str) -> gstreamer::Bin {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        let fakesinkbin = gstreamer::parse_bin_from_description(
            &format!(
                "queue name={uuid}_fakesink ! fakesink sync=false",
                uuid = uuid
            ),
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
        fakesink_pad.set_active(true).unwrap();
        fakesinkbin.add_pad(&fakesink_pad).unwrap();

        pipeline_gstreamer.pipeline.add(&fakesinkbin).unwrap();

        fakesinkbin
    }

    fn create_teeadapter(&self, uuid: &str) -> gstreamer::Bin {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

        let teebin = gstreamer::parse_bin_from_description(
            &format!("tee name={uuid}_tee", uuid = uuid),
            false,
        )
        .unwrap();

        let tee = teebin.get_by_name(&format!("{}_tee", uuid)).unwrap();

        let tee_sink = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_sink", uuid)),
            &tee.get_static_pad("sink").unwrap(),
        )
        .unwrap();
        tee_sink.set_active(true).unwrap();
        teebin.add_pad(&tee_sink).unwrap();

        let tee_src = gstreamer::GhostPad::with_target(
            Some(&format!("{}_tee_src", uuid)),
            &tee.get_request_pad("src_%u").unwrap(),
        )
        .unwrap();
        tee_src.set_active(true).unwrap();
        teebin.add_pad(&tee_src).unwrap();

        pipeline_gstreamer.pipeline.add(&teebin).unwrap();

        teebin
    }

    fn play_pipeline(&self) {
        let pipeline_gstreamer = self.pipeline_gstreamer.lock().unwrap();

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

        let fakeaudio = self.create_fakeaudio(&uuid);
        let webrtcbin = self.create_webrtcbin(&uuid);
        let tee = self.create_teeadapter(&uuid);
        let fakesink = self.create_fakesink(&uuid);
        fakeaudio.link(&webrtcbin).unwrap();

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
        uuid: &str,
        teebin_from_uuid_src: &gstreamer::Bin,
    ) -> webrtc::UserPipeline {
        let role = webrtc::Role::Consumer;

        let fakeaudio = self.create_fakeaudio(&uuid);
        let webrtcbin = self.create_webrtcbin(&uuid);
        let tee = self.create_teeadapter(&uuid);
        let fakesink = self.create_fakesink(&uuid);

        fakeaudio.sync_state_with_parent().unwrap();
        webrtcbin.sync_state_with_parent().unwrap();
        tee.sync_state_with_parent().unwrap();
        fakesink.sync_state_with_parent().unwrap();

        let uuid_split: Vec<&str> = uuid.split("_sink:").collect();
        let uuid_src = uuid_split[0][4..].to_string();

        let tee_src = teebin_from_uuid_src
            .get_by_name(&format!("{}_tee", uuid_src))
            .unwrap();

        let audio_src_pad = tee_src.get_request_pad("src_%u").unwrap();
        let audio_block = audio_src_pad
            .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                gstreamer::PadProbeReturn::Ok
            })
            .unwrap();

        let queue = gstreamer::ElementFactory::make("queue", None).unwrap();
        teebin_from_uuid_src.add(&queue).unwrap();
        queue.sync_state_with_parent().unwrap();
        let queue_sink_pad = queue.get_static_pad("sink").unwrap();
        audio_src_pad.link(&queue_sink_pad).unwrap();
        let queue_src_pad = queue.get_static_pad("src").unwrap();

        let teesrc_pad = gstreamer::GhostPad::with_target(None, &queue_src_pad).unwrap();
        teesrc_pad.set_active(true).unwrap();
        teebin_from_uuid_src.add_pad(&teesrc_pad).unwrap();
        teebin_from_uuid_src.link(&webrtcbin).unwrap();
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

    fn started(&mut self, context: &mut Self::Context) {
        context.run_interval(Duration::from_millis(5000), |this, _ctx| {
            Arbiter::spawn(Channel::heartbeat(
                this.users.clone(),
                this.pipeline_gstreamer.clone(),
            ));
        });
    }
}

impl Handler<webrtc::RequestPair> for Channel {
    type Result = ();

    fn handle(&mut self, user: webrtc::RequestPair, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [UUID: {}] [TARGET: {}] [GET PAIR REQUEST FROM CHANNEL]",
            user.room_name, user.from_uuid, user.uuid
        );
        let users = self.users.lock().unwrap();
        let mut peers = self.peers.lock().unwrap();

        if let Some(user_src) = users.get(&user.from_uuid) {
            let peer_key = format!("src:{}_sink:{}", user.from_uuid, user.uuid);
            let user_pipeline = self.build_consumer(&peer_key, &user_src.pipeline.tee);
            let new_user = user::User::new(
                user_src.room_address.clone(),
                &user.room_name,
                &peer_key,
                user_pipeline,
            )
            .unwrap();
            peers.insert(peer_key, new_user);
        }
        drop(peers);
        self.play_pipeline();
        let mut peers = self.peers.lock().unwrap();
        if let Some(user_src) = users.get(&user.uuid) {
            let peer_key = format!("src:{}_sink:{}", user.uuid, user.from_uuid);
            let user_pipeline = self.build_consumer(&peer_key, &user_src.pipeline.tee);
            let new_user = user::User::new(
                user_src.room_address.clone(),
                &user.room_name,
                &peer_key,
                user_pipeline,
            )
            .unwrap();
            peers.insert(peer_key, new_user);
        }
        drop(peers);
        self.play_pipeline();
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

        self.play_pipeline();
    }
}

impl Handler<webrtc::SessionDescription> for Channel {
    type Result = ();

    fn handle(&mut self, sdp: webrtc::SessionDescription, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [FROM UUID: {}] [TO UUID: {}] [GET SDP FROM CHANNEL]",
            sdp.room_name, sdp.from_uuid, sdp.uuid
        );

        if sdp.from_uuid != sdp.uuid {
            if sdp.uuid.contains("_sink") {
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&sdp.uuid) {
                    peer.set_session_to_gstreamer(sdp.sdp);
                }
            }
        } else {
            let users = self.users.lock().unwrap();
            if let Some(user) = users.get(&sdp.from_uuid) {
                user.set_session_to_gstreamer(sdp.sdp);
            }
        }
    }
}

impl Handler<webrtc::ICECandidate> for Channel {
    type Result = ();

    fn handle(&mut self, ice: webrtc::ICECandidate, _: &mut actix::Context<Self>) {
        info!(
            "[ROOM: {}] [FROM UUID: {}] [TO UUID: {}] [GET ICE FROM CHANNEL]",
            ice.room_name, ice.from_uuid, ice.uuid
        );

        if ice.from_uuid != ice.uuid {
            if ice.uuid.contains("_sink") {
                let peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get(&ice.uuid) {
                    peer.set_ice_to_gstreamer(ice.sdp_mline_index, ice.candidate);
                }
            }
        } else {
            let users = self.users.lock().unwrap();
            if let Some(user) = users.get(&ice.from_uuid) {
                user.set_ice_to_gstreamer(ice.sdp_mline_index, ice.candidate);
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
