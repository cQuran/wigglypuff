use crate::models::{message_websocket, webrtc};
use crate::service::room as service_room;
use actix::Addr;
use anyhow::{Context, Error};
use gstreamer;
use gstreamer::{prelude::ObjectExt, ElementExt, GstBinExt, GstObjectExt, PadExt, PadExtManual};
use log::info;
use std::sync::{Arc, Mutex, Weak};

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

pub struct ReceiverInner {
    pub webrtcbin: gstreamer::Element,
    pub room_address: Addr<service_room::Room>,
    pub room_name: String,
    pub uuid: String,
    pub pipeline: Arc<Mutex<webrtc::GstreamerPipeline>>,
}

#[derive(Clone)]
pub struct Receiver(pub Arc<ReceiverInner>);

#[derive(Clone)]
pub struct ReceiverWeak(pub Weak<ReceiverInner>);

impl std::ops::Deref for Receiver {
    type Target = ReceiverInner;

    fn deref(&self) -> &ReceiverInner {
        &self.0
    }
}

impl ReceiverWeak {
    pub fn upgrade_to_strong_reference(&self) -> Option<Receiver> {
        self.0.upgrade().map(Receiver)
    }
}

impl Receiver {
    pub fn new(
        room_address: Addr<service_room::Room>,
        requst_room_name: &String,
        request_uuid: &String,
        pipeline: Arc<Mutex<webrtc::GstreamerPipeline>>,
        webrtcbin: gstreamer::Element,
    ) -> Result<Self, Error> {
        info!(
            "[ROOM: {}] [UUID: {}] Creating WebRTC Receiver Instance",
            requst_room_name, request_uuid
        );

        let uuid = request_uuid.to_string();
        let room_name = requst_room_name.to_string();
        let receiver = Receiver(Arc::new(ReceiverInner {
            webrtcbin,
            room_address,
            room_name,
            uuid,
            pipeline,
        }));

        let receiver_clone = receiver.downgrade_to_weak_reference();
        receiver
            .webrtcbin
            .connect_pad_added(move |_webrtc, webrtc_pad| {
                let receiver = upgrade_app_weak_reference!(receiver_clone);
                receiver.on_incoming_stream(webrtc_pad);
            });

        let receiver_clone = receiver.downgrade_to_weak_reference();
        receiver
            .webrtcbin
            .connect("on-negotiation-needed", false, move |_values| {
                let receiver = upgrade_app_weak_reference!(receiver_clone, None);
                receiver.on_negotiation_needed();
                None
            })
            .unwrap();

        let receiver_clone = receiver.downgrade_to_weak_reference();
        receiver
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

                let receiver = upgrade_app_weak_reference!(receiver_clone, None);
                receiver.on_ice_candidate(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();
        receiver
            .webrtcbin
            .connect_notify(Some("connection-state"), move |webrtcbin, _spec| {
                let connection = webrtcbin
                    .get_property("connection-state")
                    .unwrap()
                    .get::<gstreamer_webrtc::WebRTCPeerConnectionState>()
                    .unwrap();
                info!(
                    "[ROOM: {}] [UUID: {}] [CONNECTION STATE: {:#?}]",
                    room_name_copy,
                    uuid_copy,
                    connection.unwrap()
                );
            });

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();
        receiver
            .webrtcbin
            .connect_notify(Some("ice-connection-state"), move |webrtcbin, _spec| {
                let ice_connection = webrtcbin
                    .get_property("ice-connection-state")
                    .unwrap()
                    .get::<gstreamer_webrtc::WebRTCICEConnectionState>()
                    .unwrap();
                info!(
                    "[ROOM: {}] [UUID: {}] [ICE CONNECTION STATE: {:#?}]",
                    room_name_copy,
                    uuid_copy,
                    ice_connection.unwrap()
                );
            });

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();
        receiver
            .webrtcbin
            .connect_notify(Some("ice-gathering-state"), move |webrtcbin, _spec| {
                let gather_connection = webrtcbin
                    .get_property("ice-gathering-state")
                    .unwrap()
                    .get::<gstreamer_webrtc::WebRTCICEGatheringState>()
                    .unwrap();
                info!(
                    "[ROOM: {}] [UUID: {}] [GATHER CONNECTION STATE: {:#?}]",
                    room_name_copy,
                    uuid_copy,
                    gather_connection.unwrap()
                );
            });

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();
        receiver
            .webrtcbin
            .connect_notify(Some("signaling-state"), move |webrtcbin, _spec| {
                let signalling = webrtcbin
                    .get_property("signaling-state")
                    .unwrap()
                    .get::<gstreamer_webrtc::WebRTCSignalingState>()
                    .unwrap();
                info!(
                    "[ROOM: {}] [UUID: {}] [SIGNALLING STATE: {:#?}]",
                    room_name_copy,
                    uuid_copy,
                    signalling.unwrap()
                );
            });
        Ok(receiver)
    }

    pub fn downgrade_to_weak_reference(&self) -> ReceiverWeak {
        ReceiverWeak(Arc::downgrade(&self.0))
    }

    pub fn on_session_answer(&self, session_description_request: String) {
        let ret = gstreamer_sdp::SDPMessage::parse_buffer(session_description_request.as_bytes())
            .map_err(|_| info!("Failed to parse SDP offer"))
            .unwrap();

        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(
            gstreamer_webrtc::WebRTCSDPType::Answer,
            ret,
        );

        self.webrtcbin
            .emit(
                "set-remote-description",
                &[&answer, &None::<gstreamer::Promise>],
            )
            .unwrap();
    }

    pub fn on_ice_answer(&self, sdp_mline_index: u32, candidate: String) {
        self.webrtcbin
            .emit("add-ice-candidate", &[&sdp_mline_index, &candidate])
            .unwrap();
    }

    fn on_negotiation_needed(&self) {
        info!(
            "[ROOM: {}] [UUID: {}] [WEBRTC] [STARTING NEGOTIATION]",
            self.room_name, self.uuid
        );
        let receiver_clone = self.downgrade_to_weak_reference();
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let receiver = upgrade_app_weak_reference!(receiver_clone);
            receiver.on_offer_created(reply);
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
            info!("LINKED");
            let pipeline_gstreamer = self.pipeline.lock().unwrap();

            let decodebin = gstreamer::ElementFactory::make(
                "decodebin",
                Some(&format!("{}_decodebin", self.uuid.clone(),)),
            )
            .unwrap();

            let receiver_clone = self.downgrade_to_weak_reference();
            let room = self.room_name.clone();
            let uuid = self.uuid.clone();
            decodebin.connect_pad_added(move |_decodebin, source_pad| {
                let receiver = upgrade_app_weak_reference!(receiver_clone);
                info!(
                    "[ROOM: {}] [UUID: {}] [PAD ADDED DECODEBIN] {}",
                    room,
                    uuid,
                    source_pad.get_name()
                );
                receiver.on_incoming_decodebin_stream(source_pad);
            });

            pipeline_gstreamer.pipeline.add(&decodebin).unwrap();
            decodebin.sync_state_with_parent().unwrap();
            let sinkpad = decodebin.get_static_pad("sink").unwrap();
            webrtc_source_pad.link(&sinkpad).unwrap();
        }
    }

    fn on_incoming_decodebin_stream(&self, decodebin_source_pad: &gstreamer::Pad) {
        let caps = decodebin_source_pad.get_current_caps().unwrap();
        let name = caps.get_structure(0).unwrap().get_name();

        let next = if name.starts_with("audio/") {
            gstreamer::parse_bin_from_description(
                &format!(
                    "queue ! audioconvert ! audioresample name={}_audioresample ! autoaudiosink name={}_autoaudiosink",
                    self.uuid, self.uuid
                ),
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

        let pipeline_gstreamer = self.pipeline.lock().unwrap();
        pipeline_gstreamer.pipeline.add(&next).unwrap();

        next.sync_state_with_parent()
            .with_context(|| format!("can't start sink for stream {:?}", caps))
            .unwrap();

        let next_pad = next.get_static_pad("sink").unwrap();

        decodebin_source_pad
            .link(&next_pad)
            .with_context(|| format!("can't link sink for stream {:?}", caps))
            .unwrap();

        info!(
            "[ROOM: {}] [UUID: {}] [SINK] [AUDIO SUCCESS]",
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
