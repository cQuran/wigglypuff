use crate::models::{message_websocket, webrtc};
use crate::service::room as service_room;
use actix::Addr;
use anyhow::Error;
use gstreamer;
use gstreamer::{prelude::ObjectExt, ElementExt, GstBinExt, GstBinExtManual, PadExt, PadExtManual};
use log::info;
use std::collections::BTreeMap;
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
    pub peer_audiomixer: Arc<Mutex<BTreeMap<String, (gstreamer::Pad, gstreamer::Element)>>>,
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
        peer_audiomixer: Arc<Mutex<BTreeMap<String, (gstreamer::Pad, gstreamer::Element)>>>,
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
            peer_audiomixer,
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

            let rtpopusdepay = gstreamer::ElementFactory::make(
                "rtpopusdepay",
                Some(&format!("{}_rtpopusdepay", self.uuid)),
            )
            .unwrap();

            let opusdec =
                gstreamer::ElementFactory::make("opusdec", Some(&format!("{}_opusdec", self.uuid)))
                    .unwrap();

            let audioconvert = gstreamer::ElementFactory::make(
                "audioconvert",
                Some(&format!("{}_audioconvert_rtpopusdepay", self.uuid)),
            )
            .unwrap();

            let audioresample = gstreamer::ElementFactory::make(
                "audioresample",
                Some(&format!("{}_audioresample_rtpopusdepay", self.uuid)),
            )
            .unwrap();

            pipeline_gstreamer
                .pipeline
                .add_many(&[&rtpopusdepay, &opusdec, &audioconvert, &audioresample])
                .unwrap();
            rtpopusdepay.sync_state_with_parent().unwrap();
            opusdec.sync_state_with_parent().unwrap();
            audioconvert.sync_state_with_parent().unwrap();
            audioresample.sync_state_with_parent().unwrap();

            let rtpopusdepay_sink = rtpopusdepay.get_static_pad("sink").unwrap();
            webrtc_source_pad.link(&rtpopusdepay_sink).unwrap();

            let audiomixer = pipeline_gstreamer
                .pipeline
                .get_by_name(&format!("{}_audiomixer", self.uuid))
                .expect("can't find webrtcbin");

            let rtpopusdepay_src = rtpopusdepay.get_static_pad("src").unwrap();
            let opusdec_sink = opusdec.get_static_pad("sink").unwrap();
            let opusdec_src = opusdec.get_static_pad("src").unwrap();
            rtpopusdepay_src.link(&opusdec_sink).unwrap();
            let audioconvert_sink = audioconvert.get_static_pad("sink").unwrap();
            opusdec_src.link(&audioconvert_sink).unwrap();
            let audioconvert_src = audioconvert.get_static_pad("src").unwrap();
            let audioresample_sink = audioresample.get_static_pad("sink").unwrap();
            let audioresample_src = audioresample.get_static_pad("src").unwrap();
            audioconvert_src.link(&audioresample_sink).unwrap();
            info!("INI LHO TEEE AWAL {}", self.uuid);

            let tee = gstreamer::ElementFactory::make("tee", Some(&format!("{}_tee", self.uuid)))
                .unwrap();

            pipeline_gstreamer.pipeline.add_many(&[&tee]).unwrap();
            tee.sync_state_with_parent().unwrap();
            let tee_sink = tee.get_static_pad("sink").unwrap();
            audioresample_src.link(&tee_sink).unwrap();

            let tee_src = tee.get_request_pad("src_%u").unwrap();
            let audiomixer_sink = audiomixer.get_request_pad("sink_%u").unwrap();
            tee_src.link(&audiomixer_sink).unwrap();

            let mut peer = self.peer_audiomixer.lock().unwrap();

            for (key, (pad, audiomixer_target)) in peer.iter() {
                let tee_src = tee.get_request_pad("src_%u").unwrap();
                let tee_block = tee_src
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();
                let audiomixer_target_pad = audiomixer_target.get_request_pad("sink_%u").unwrap();
                let audiomixer_block = audiomixer_target_pad
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();
                tee_src.link(&audiomixer_target_pad).unwrap();
                tee_src.remove_probe(tee_block);
                audiomixer_target_pad.remove_probe(audiomixer_block);

                let audiomixer_source_pad = audiomixer.get_request_pad("sink_%u").unwrap();
                let audiomixer_block = audiomixer_source_pad
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();
                let tee_target = pipeline_gstreamer
                    .pipeline
                    .get_by_name(&format!("{}_tee", key))
                    .expect("can't find webrtcbin");
                let tee_src = tee_target.get_request_pad("src_%u").unwrap();
                let tee_block = tee_src
                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                        gstreamer::PadProbeReturn::Ok
                    })
                    .unwrap();
                tee_src.link(&audiomixer_source_pad).unwrap();
                tee_src.remove_probe(tee_block);
                audiomixer_source_pad.remove_probe(audiomixer_block);
            }

            peer.insert(self.uuid.clone(), (webrtc_source_pad.clone(), audiomixer));

            info!("TEEE KETAMBAH");
        }
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
