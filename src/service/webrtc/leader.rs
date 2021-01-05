use crate::service::room as service_room;
use crate::{
    constants,
    models::{message_websocket, webrtc},
};
use actix::Addr;
use anyhow::{Context, Error};
use gstreamer;
use gstreamer::{
    prelude::{Cast, ObjectExt},
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstObjectExt, PadExt,
    PadExtManual,
};

use log::info;
use std::sync::{Arc, Weak};

macro_rules! upgrade_app_weak_reference {
    ($x:ident, $r:expr) => {{
        match $x.upgrade_to_strong_reference() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_app_weak_reference!($x, ())
    };
}

pub struct LeaderInner {
    pub pipeline: gstreamer::Pipeline,
    pub webrtcbin: gstreamer::Element,
    pub room_address: Addr<service_room::Room>,
    pub room_name: String,
    pub uuid: String,
}

#[derive(Clone)]
pub struct Leader(pub Arc<LeaderInner>);

#[derive(Clone)]
pub struct LeaderWeak(pub Weak<LeaderInner>);

impl std::ops::Deref for Leader {
    type Target = LeaderInner;

    fn deref(&self) -> &LeaderInner {
        &self.0
    }
}

impl LeaderWeak {
    pub fn upgrade_to_strong_reference(&self) -> Option<Leader> {
        self.0.upgrade().map(Leader)
    }
}

impl Leader {
    pub fn new(
        room_address: &Addr<service_room::Room>,
        room_name: &String,
        uuid: &String,
    ) -> Result<Self, Error> {
        info!(
            "[ROOM: {}] [UUID: {}] Creating WebRTC Leader Instance",
            room_name, uuid
        );

        let pipeline = gstreamer::parse_launch(
            "audiotestsrc is-live=true ! opusenc ! rtpopuspay pt=97 ! webrtcbin. \
             webrtcbin name=webrtcbin ! rtpopusdepay ! queue leaky=2 ! rtpopuspay pt=97 ! tee name=audio-tee",
        )
        .unwrap();

        let pipeline = pipeline
            .downcast::<gstreamer::Pipeline>()
            .expect("not a pipeline");

        let webrtcbin = pipeline
            .get_by_name("webrtcbin")
            .expect("can't find webrtcbin");

        webrtcbin.set_property_from_str("stun-server", constants::STUN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        let room_name = room_name.clone();
        let uuid = uuid.clone();
        let room_address = room_address.clone();

        let leader = Leader(Arc::new(LeaderInner {
            pipeline,
            webrtcbin,
            room_address,
            room_name,
            uuid,
        }));

        let leader_clone = leader.downgrade_to_weak_reference();
        leader
            .webrtcbin
            .connect_pad_added(move |_webrtc, webrtc_pad| {
                let leader = upgrade_app_weak_reference!(leader_clone);

                info!("PAD ADDEDDD WEBRTC {}", webrtc_pad.get_name());
                leader.on_incoming_stream(webrtc_pad);
            });

        let leader_clone = leader.downgrade_to_weak_reference();
        leader
            .webrtcbin
            .connect("on-negotiation-needed", false, move |_values| {
                let leader = upgrade_app_weak_reference!(leader_clone, None);
                leader.on_negotiation_needed();
                None
            })
            .unwrap();

        let leader_clone = leader.downgrade_to_weak_reference();
        leader
            .webrtcbin
            .connect("on-ice-candidate", false, move |values| {
                let _webrtc = values[0]
                    .get::<gstreamer::Element>()
                    .expect("Invalid argument");
                let sdp_mline_index = values[1].get_some::<u32>().expect("Invalid argument");
                let candidate = values[2]
                    .get::<String>()
                    .expect("Invalid argument")
                    .unwrap();

                let leader = upgrade_app_weak_reference!(leader_clone, None);
                leader.on_ice_candidate(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

        leader
            .webrtcbin
            .connect_notify(Some("connection-state"), |webrtcbin, _spec| {
                info!(
                    "[CONNECTION STATE: {:#?}]",
                    webrtcbin
                        .get_property("connection-state")
                        .unwrap()
                        .get::<gstreamer_webrtc::WebRTCPeerConnectionState>()
                        .unwrap()
                );
            });

        leader
            .webrtcbin
            .connect_notify(Some("ice-connection-state"), |webrtcbin, _spec| {
                info!(
                    "[ICE CONNECTION STATE: {:#?}]",
                    webrtcbin
                        .get_property("ice-connection-state")
                        .unwrap()
                        .get::<gstreamer_webrtc::WebRTCICEConnectionState>()
                        .unwrap()
                );
            });

        leader
            .webrtcbin
            .connect_notify(Some("ice-gathering-state"), |webrtcbin, _spec| {
                info!(
                    "[GATHER CONNECTION STATE: {:#?}]",
                    webrtcbin
                        .get_property("ice-gathering-state")
                        .unwrap()
                        .get::<gstreamer_webrtc::WebRTCICEGatheringState>()
                        .unwrap()
                );
            });

        leader
            .webrtcbin
            .connect_notify(Some("signaling-state"), |webrtcbin, _spec| {
                info!(
                    "[SIGNALLING STATE: {:#?}]",
                    webrtcbin
                        .get_property("signaling-state")
                        .unwrap()
                        .get::<gstreamer_webrtc::WebRTCSignalingState>()
                        .unwrap()
                );
            });

        leader.pipeline.call_async(|pipeline| {
            if pipeline.set_state(gstreamer::State::Playing).is_err() {
                info!("Failed to set pipeline to Playing");
            }
        });

        leader.pipeline.call_async(|pipeline| {
            pipeline
                .set_state(gstreamer::State::Playing)
                .expect("Couldn't set pipeline to Playing");
        });

        Ok(leader)
    }

    pub fn downgrade_to_weak_reference(&self) -> LeaderWeak {
        LeaderWeak(Arc::downgrade(&self.0))
    }

    fn on_negotiation_needed(&self) {
        info!(
            "[WEBRTC] [ROOM: {}] [UUID: {}] [STARTING NEGOTIATION]",
            self.room_name, self.uuid
        );
        let leader_clone = self.downgrade_to_weak_reference();
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let leader = upgrade_app_weak_reference!(leader_clone);
            leader.on_offer_created(reply);
        });

        self.webrtcbin
            .emit("create-offer", &[&None::<gstreamer::Structure>, &promise])
            .unwrap();
    }

    fn on_ice_candidate(&self, candidate: &String, sdp_mline_index: &u32) {
        self.room_address.do_send(webrtc::WigglypuffWebRTC::new(
            &self.uuid,
            &self.room_name,
            message_websocket::MessageSocketType::ICECandidate {
                candidate: candidate.to_owned(),
                sdp_mline_index: sdp_mline_index.to_owned(),
            },
        ));
    }

    fn on_incoming_stream(&self, webrtc_source_pad: &gstreamer::Pad) {
        if webrtc_source_pad.get_direction() == gstreamer::PadDirection::Src {
            let decodebin =
                gstreamer::ElementFactory::make("decodebin", Some("decodebin_from_audio")).unwrap();
            let leader_clone = self.downgrade_to_weak_reference();
            decodebin.connect_pad_added(move |_decodebin, source_pad| {
                let leader = upgrade_app_weak_reference!(leader_clone);
                info!("PAD ADDEDDD DECODEBIN {}", source_pad.get_name());
                leader.on_incoming_decodebin_stream(source_pad);
            });

            self.pipeline.add(&decodebin).unwrap();
            decodebin.sync_state_with_parent().unwrap();

            // TODO
            // webrtc_source_pad.link(&decodebin_sink_pad).unwrap();
        }
    }

    fn on_incoming_decodebin_stream(&self, decodebin_source_pad: &gstreamer::Pad) {
        let caps = decodebin_source_pad.get_current_caps().unwrap();
        let name = caps.get_structure(0).unwrap().get_name();

        let queue_sink = if name.starts_with("video/") {
            info!("[WARN] VIDEO SINK");
            gstreamer::parse_bin_from_description(
                "queue ! videoconvert ! videoscale ! autovideosink",
                true,
            )
            .unwrap()
        } else if name.starts_with("audio/") {
            gstreamer::parse_bin_from_description(
                "queue ! audioconvert ! audioresample ! autoaudiosink",
                true,
            )
            .unwrap()
        } else {
            info!("[WARN] VIDEO SINK");
            gstreamer::parse_bin_from_description(
                "queue ! videoconvert ! videoscale ! autovideosink",
                true,
            )
            .unwrap()
        };

        self.pipeline.add(&queue_sink).unwrap();
        queue_sink
            .sync_state_with_parent()
            .with_context(|| format!("can't start sink for stream {:?}", caps))
            .unwrap();

        let queue_sink_pad = queue_sink.get_static_pad("sink").unwrap();
        decodebin_source_pad
            .link(&queue_sink_pad)
            .with_context(|| format!("can't link sink for stream {:?}", caps))
            .unwrap();

        info!(
            "[SINK] [ROOM: {}] [UUID: {}] AUDIO SUCCESS",
            self.room_name, self.uuid
        );
    }

    fn on_offer_created(
        &self,
        reply: Result<Option<&gstreamer::StructureRef>, gstreamer::PromiseError>,
    ) {
        match reply {
            Ok(Some(reply)) => {
                let offer = reply
                    .get_value("offer")
                    .unwrap()
                    .get::<gstreamer_webrtc::WebRTCSessionDescription>()
                    .expect("Invalid argument")
                    .unwrap();

                self.webrtcbin
                    .emit(
                        "set-local-description",
                        &[&offer, &None::<gstreamer::Promise>],
                    )
                    .unwrap();

                self.room_address.do_send(webrtc::WigglypuffWebRTC::new(
                    &self.uuid,
                    &self.room_name,
                    message_websocket::MessageSocketType::SessionDescription {
                        types: "offer".to_string(),
                        sdp: offer.get_sdp().as_text().unwrap(),
                    },
                ));
            }
            Ok(None) => {
                info!("Offer creation future got no reponse");
            }
            Err(err) => {
                info!("Offer creation future got error reponse: {:?}", err);
            }
        };
    }
}

impl Drop for LeaderInner {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}
