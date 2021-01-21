use crate::models::{message_websocket, webrtc};
use crate::service::room as service_room;
use actix::Addr;
use anyhow::Error;
use glib::ToValue;
use gstreamer;
use gstreamer::{prelude::ObjectExt, GstBinExt};
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

pub struct UserInner {
    pub room_address: Addr<service_room::Room>,
    pub room_name: String,
    pub uuid: String,
    pub pipeline: webrtc::UserPipeline
}

#[derive(Clone)]
pub struct User(pub Arc<UserInner>);

#[derive(Clone)]
pub struct UserWeak(pub Weak<UserInner>);

impl std::ops::Deref for User {
    type Target = UserInner;

    fn deref(&self) -> &UserInner {
        &self.0
    }
}

impl UserWeak {
    pub fn upgrade_to_strong_reference(&self) -> Option<User> {
        self.0.upgrade().map(User)
    }
}

impl User {
    pub fn new(
        room_address: Addr<service_room::Room>,
        requst_room_name: &String,
        request_uuid: &String,
        pipeline: webrtc::UserPipeline,
    ) -> Result<Self, Error> {
        info!(
            "[ROOM: {}] [UUID: {}] Creating WebRTC User Instance",
            requst_room_name, request_uuid
        );

        let uuid = request_uuid.to_string();
        let room_name = requst_room_name.to_string();
        let user = User(Arc::new(UserInner {
            room_address,
            room_name,
            uuid,
            pipeline,
        }));

        let user_clone = user.downgrade_to_weak_reference();
        let webrtcbin = user
            .pipeline
            .webrtcbin
            .get_by_name(&format!("{}_webrtcbin", request_uuid))
            .expect("can't find webrtcbin");

        webrtcbin
            .connect("on-negotiation-needed", false, move |_values| {
                let user = upgrade_app_weak_reference!(user_clone, None);
                user.on_negotiation_needed();
                None
            })
            .unwrap();

        let user_clone = user.downgrade_to_weak_reference();
        webrtcbin
            .connect("on-ice-candidate", false, move |values| {
                let sdp_mline_index = values[1].get_some::<u32>().expect("Invalid argument");
                let candidate = values[2]
                    .get::<String>()
                    .expect("Invalid argument")
                    .unwrap();

                let user = upgrade_app_weak_reference!(user_clone, None);
                user.on_ice_candidate(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();

        let user_clone = user.downgrade_to_weak_reference();
        webrtcbin.connect_notify(Some("connection-state"), move |webrtcbin, _| {
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
            if connection.unwrap() == gstreamer_webrtc::WebRTCPeerConnectionState::Connected {
                let user = upgrade_app_weak_reference!(user_clone);
                match user.pipeline.role {
                    webrtc::Role::Consumer {} => {
                        info!("INI CONSUMER");
                    }
                    webrtc::Role::Producer {} => {
                        user.room_address.do_send(webrtc::WigglypuffWebRTC::new(
                            &uuid_copy,
                            &room_name_copy,
                            user.pipeline.role.clone(),
                            message_websocket::MessageSocketType::WebRTCConnectionState,
                        ));
                    }
                };
            }
        });

        let room_name_copy = requst_room_name.to_string();
        let uuid_copy = request_uuid.to_string();
        webrtcbin.connect_notify(Some("ice-connection-state"), move |webrtcbin, _| {
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
        webrtcbin.connect_notify(Some("ice-gathering-state"), move |webrtcbin, _| {
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
        webrtcbin.connect_notify(Some("signaling-state"), move |webrtcbin, _| {
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
        Ok(user)
    }

    pub fn downgrade_to_weak_reference(&self) -> UserWeak {
        UserWeak(Arc::downgrade(&self.0))
    }

    pub fn on_session_answer(&self, session_description_request: String) {
        let ret = gstreamer_sdp::SDPMessage::parse_buffer(session_description_request.as_bytes())
            .map_err(|_| info!("Failed to parse SDP offer"))
            .unwrap();

        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(
            gstreamer_webrtc::WebRTCSDPType::Answer,
            ret,
        );

        let webrtcbin = self
            .pipeline
            .webrtcbin
            .get_by_name(&format!("{}_webrtcbin", self.uuid))
            .expect("can't find webrtcbin");

        webrtcbin
            .emit(
                "set-remote-description",
                &[&answer, &None::<gstreamer::Promise>],
            )
            .unwrap();
    }

    pub fn on_ice_answer(&self, sdp_mline_index: u32, candidate: String) {
        let webrtcbin = self
            .pipeline
            .webrtcbin
            .get_by_name(&format!("{}_webrtcbin", self.uuid))
            .expect("can't find webrtcbin");

        webrtcbin
            .emit("add-ice-candidate", &[&sdp_mline_index, &candidate])
            .unwrap();
    }

    fn on_negotiation_needed(&self) {
        info!(
            "[ROOM: {}] [UUID: {}] [WEBRTC] [STARTING NEGOTIATION]",
            self.room_name, self.uuid
        );
        let user_clone = self.downgrade_to_weak_reference();
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let user = upgrade_app_weak_reference!(user_clone);
            user.on_offer_created(reply);
        });

        let webrtcbin = self
            .pipeline
            .webrtcbin
            .get_by_name(&format!("{}_webrtcbin", self.uuid))
            .expect("can't find webrtcbin");

        if let Ok(transceiver) = webrtcbin.emit("get-transceiver", &[&0.to_value()]) {
            if let Some(t) = transceiver {
                if let Ok(obj) = t.get::<glib::Object>() {
                    let role = match self.pipeline.role {
                        webrtc::Role::Consumer {} => {
                            gstreamer_webrtc::WebRTCRTPTransceiverDirection::Sendonly
                        }
                        webrtc::Role::Producer {} => {
                            gstreamer_webrtc::WebRTCRTPTransceiverDirection::Recvonly
                        }
                    };
                    obj.expect("Error set Transceiver")
                        .set_property("direction", &role)
                        .unwrap();
                }
            }
        }

        webrtcbin
            .emit("create-offer", &[&None::<gstreamer::Structure>, &promise])
            .unwrap();
    }

    fn on_ice_candidate(&self, candidate: &String, sdp_mline_index: &u32) {
        self.room_address.do_send(webrtc::WigglypuffWebRTC::new(
            &self.uuid,
            &self.room_name,
            self.pipeline.role.clone(),
            message_websocket::MessageSocketType::ICECandidate {
                uuid: self.uuid.to_string(),
                candidate: candidate.to_owned(),
                sdp_mline_index: sdp_mline_index.to_owned(),
            },
        ));
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

                let webrtcbin = self
                    .pipeline
                    .webrtcbin
                    .get_by_name(&format!("{}_webrtcbin", self.uuid))
                    .expect("can't find webrtcbin");

                webrtcbin
                    .emit(
                        "set-local-description",
                        &[&offer, &None::<gstreamer::Promise>],
                    )
                    .unwrap();

                self.room_address.do_send(webrtc::WigglypuffWebRTC::new(
                    &self.uuid,
                    &self.room_name,
                    self.pipeline.role.clone(),
                    message_websocket::MessageSocketType::SessionDescription {
                        uuid: self.uuid.to_string(),
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
