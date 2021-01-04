use crate::service::room as service_room;
use crate::{
    constants,
    models::{message_websocket, webrtc},
};
use actix::{Addr};
use anyhow::{Context, Error};
use gstreamer;
use gstreamer::{
    prelude::{Cast, ObjectExt},
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, PadExt, PadExtManual,
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
             webrtcbin name=webrtcbin",
        )
        .unwrap();

        let pipeline = pipeline
            .downcast::<gstreamer::Pipeline>()
            .expect("not a pipeline");

        let webrtcbin = pipeline
            .get_by_name("webrtcbin")
            .expect("can't find webrtcbin");

        webrtcbin.set_property_from_str("stun-server", constants::STUN_SERVER);
        // webrtcbin.set_property_from_str("turn-server", constants::TURN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        let room_name = room_name.clone();
        let uuid = uuid.clone();
        let room_address = room_address.clone();

        let app = Leader(Arc::new(LeaderInner {
            pipeline,
            webrtcbin,
            room_address,
            room_name,
            uuid,
        }));

        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin.connect_pad_added(move |_webrtc, pad| {
            let app = upgrade_app_weak_reference!(app_clone);
            app.on_incoming_stream(pad);
        });

        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin
            .connect("on-negotiation-needed", false, move |_values| {
                let app = upgrade_app_weak_reference!(app_clone, None);
                app.on_negotiation_needed();
                None
            })
            .unwrap();

        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin
            .connect("on-ice-candidate", false, move |values| {
                let _webrtc = values[0]
                    .get::<gstreamer::Element>()
                    .expect("Invalid argument");
                let sdp_mline_index = values[1].get_some::<u32>().expect("Invalid argument");
                let candidate = values[2]
                    .get::<String>()
                    .expect("Invalid argument")
                    .unwrap();

                let app = upgrade_app_weak_reference!(app_clone, None);
                app.on_ice_candidate(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

        app.webrtcbin
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

        app.webrtcbin
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

        app.webrtcbin
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

        app.webrtcbin
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

        app.pipeline.call_async(|pipeline| {
            if pipeline.set_state(gstreamer::State::Playing).is_err() {
                info!("Failed to set pipeline to Playing");
            }
        });

        app.pipeline.call_async(|pipeline| {
            pipeline
                .set_state(gstreamer::State::Playing)
                .expect("Couldn't set pipeline to Playing");
        });

        Ok(app)
    }

    pub fn downgrade_to_weak_reference(&self) -> LeaderWeak {
        LeaderWeak(Arc::downgrade(&self.0))
    }

    fn on_negotiation_needed(&self) {
        info!(
            "[WEBRTC] [ROOM: {}] [UUID: {}] [STARTING NEGOTIATION]",
            self.room_name, self.uuid
        );
        let app_clone = self.downgrade_to_weak_reference();
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let app = upgrade_app_weak_reference!(app_clone);
            app.on_offer_created(reply);
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

    fn on_incoming_stream(&self, pad: &gstreamer::Pad) {
        if pad.get_direction() == gstreamer::PadDirection::Src {
            let decodebin = gstreamer::ElementFactory::make("decodebin", None).unwrap();
            let app_clone = self.downgrade_to_weak_reference();
            decodebin.connect_pad_added(move |_decodebin, pad| {
                let app = upgrade_app_weak_reference!(app_clone);
                app.on_incoming_decodebin_stream(pad);
            });

            self.pipeline.add(&decodebin).unwrap();
            decodebin.sync_state_with_parent().unwrap();

            let sinkpad = decodebin.get_static_pad("sink").unwrap();
            pad.link(&sinkpad).unwrap();
        }
    }

    fn on_incoming_decodebin_stream(&self, pad: &gstreamer::Pad) {
        let caps = pad.get_current_caps().unwrap();
        let name = caps.get_structure(0).unwrap().get_name();

        let sink = if name.starts_with("video/") {
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

        self.pipeline.add(&sink).unwrap();
        sink.sync_state_with_parent()
            .with_context(|| format!("can't start sink for stream {:?}", caps))
            .unwrap();

        let sinkpad = sink.get_static_pad("sink").unwrap();
        pad.link(&sinkpad)
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