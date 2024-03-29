use crate::models::{message_websocket, network_transversal, webrtc};
use crate::service::room as service_room;

use actix::Addr;
use anyhow::Error;
use glib::ToValue;
use glib::Value;
use gstreamer;
use gstreamer::{prelude::ObjectExt, ElementExt, GstBinExt, PadExt, PadExtManual};
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
    pub pipeline: webrtc::UserPipeline,
    pub nats: Vec<network_transversal::STUNTURN>,
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
        nats: Vec<network_transversal::STUNTURN>,
    ) -> Result<Self, Error> {
        info!(
            "[ROOM: {}] [UUID: {}] [CREATING WEBRTC INSTANCE] ",
            requst_room_name, request_uuid
        );

        let uuid = request_uuid.to_string();
        let room_name = requst_room_name.to_string();
        let user = User(Arc::new(UserInner {
            room_address,
            room_name,
            uuid,
            pipeline,
            nats,
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
                user.on_offer_from_gstreamer();
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
                user.on_ice_from_gstreamer(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

        let user_clone = user.downgrade_to_weak_reference();
        webrtcbin.connect_pad_added(move |_webrtc, pad| {
            let user = upgrade_app_weak_reference!(user_clone);
            user.on_incoming_stream(pad);
        });

        Ok(user)
    }

    pub fn downgrade_to_weak_reference(&self) -> UserWeak {
        UserWeak(Arc::downgrade(&self.0))
    }

    pub fn set_session_to_gstreamer(&self, session_description_request: String) {
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

    pub fn set_ice_to_gstreamer(&self, sdp_mline_index: u32, candidate: String) {
        let webrtcbin = self
            .pipeline
            .webrtcbin
            .get_by_name(&format!("{}_webrtcbin", self.uuid))
            .expect("can't find webrtcbin");

        webrtcbin
            .emit("add-ice-candidate", &[&sdp_mline_index, &candidate])
            .unwrap();
    }

    fn on_offer_from_gstreamer(&self) {
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

        let turn_address_0 = match &self.nats[1] {
            network_transversal::STUNTURN::TURN {
                url,
                urls,
                username,
                credential,
            } => Some(url.clone().replace(
                "turn:",
                &format!(
                    "turn://{username}:{credential}@",
                    username = username,
                    credential = credential
                ),
            )),
            _ => None,
        };

        let turn_address_1 = match &self.nats[2] {
            network_transversal::STUNTURN::TURN {
                url,
                urls,
                username,
                credential,
            } => Some(url.clone().replace(
                "turn:",
                &format!(
                    "turn://{username}:{credential}@",
                    username = username,
                    credential = credential
                ),
            )),
            _ => None,
        };

        let turn_address_2 = match &self.nats[3] {
            network_transversal::STUNTURN::TURN {
                url,
                urls,
                username,
                credential,
            } => Some(url.clone().replace(
                "turn:",
                &format!(
                    "turn://{username}:{credential}@",
                    username = username,
                    credential = credential
                ),
            )),
            _ => None,
        };

        let turn_address_0 = turn_address_0.unwrap();
        let turn_address_1 = turn_address_1.unwrap();
        let turn_address_2 = turn_address_2.unwrap();
        info!("{}", turn_address_1);
        info!("{}", turn_address_2);
        webrtcbin
            .emit("add-turn-server", &[&Value::from(turn_address_0.as_str())])
            .unwrap();
        webrtcbin
            .emit("add-turn-server", &[&Value::from(turn_address_1.as_str())])
            .unwrap();
        webrtcbin
            .emit("add-turn-server", &[&Value::from(turn_address_2.as_str())])
            .unwrap();
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

    fn on_ice_from_gstreamer(&self, candidate: &String, sdp_mline_index: &u32) {
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

    fn on_incoming_stream(&self, pad: &gstreamer::Pad) {
        if pad.get_direction() == gstreamer::PadDirection::Src {
            let rtpopusdepay =
                gstreamer::ElementFactory::make("rtpopusdepay", Some("source")).unwrap();

            self.pipeline.webrtcbin.add(&rtpopusdepay).unwrap();
            rtpopusdepay.sync_state_with_parent().unwrap();
            pad.link(&rtpopusdepay.get_static_pad("sink").unwrap())
                .unwrap();

            let rtpopusdepay_src_pad = gstreamer::GhostPad::with_target(
                Some(&format!("{}_audiosrc", self.uuid)),
                &rtpopusdepay.get_static_pad("src").unwrap(),
            )
            .unwrap();
            rtpopusdepay_src_pad.set_active(true).unwrap();

            self.pipeline
                .webrtcbin
                .add_pad(&rtpopusdepay_src_pad)
                .unwrap();

            self.pipeline.webrtcbin.link(&self.pipeline.tee).unwrap();
            self.pipeline.tee.link(&self.pipeline.fakesink).unwrap();

            match self.pipeline.role {
                webrtc::Role::Consumer {} => {}
                webrtc::Role::Producer {} => {
                    self.room_address.do_send(webrtc::WigglypuffWebRTC::new(
                        &self.uuid,
                        &self.room_name,
                        self.pipeline.role.clone(),
                        message_websocket::MessageSocketType::WebRTCConnectionState,
                    ));
                }
            };
        }
    }
}
