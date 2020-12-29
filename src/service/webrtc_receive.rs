use crate::service::room as service_room;
use crate::{
    constants,
    models::{message_websocket, webrtc},
};
use actix::{Actor, Addr, Context, Handler};
use anyhow::Error;
use futures::Stream;
use gstreamer;
use gstreamer::{
    prelude::{Cast, ObjectExt},
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstBinExtManual,
};

use log::info;
use std::sync::{Arc, Weak};

#[macro_export]
macro_rules! upgrade_weak_reference {
    ($x:ident, $r:expr) => {{
        match $x.upgrade_to_strong_reference() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak_reference!($x, ())
    };
}

struct AppInner {
    pipeline: gstreamer::Pipeline,
    webrtcbin: gstreamer::Element,
    room_address: Addr<service_room::Room>,
    room_name: String,
    uuid: String,
}

#[derive(Clone)]
struct App(Arc<AppInner>);

#[derive(Clone)]
struct AppWeak(Weak<AppInner>);

impl std::ops::Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &AppInner {
        &self.0
    }
}

impl AppWeak {
    fn upgrade_to_strong_reference(&self) -> Option<App> {
        self.0.upgrade().map(App)
    }
}

impl App {
    fn new(
        room_address: &Addr<service_room::Room>,
        room_name: &String,
        uuid: &String,
    ) -> Result<(Self, impl Stream<Item = gstreamer::Message>), Error> {
        info!("Creating WebRTC Connection");
        let source_webrtcbin = gstreamer::ElementFactory::make("webrtcbin", Some("webrtcbin"))
            .expect("Could not instanciate uridecodebin");
        let pipeline = gstreamer::Pipeline::new(Some("webrtc-pipeline"));

        pipeline.add_many(&[&source_webrtcbin]).unwrap();
        let pipeline = pipeline
            .downcast::<gstreamer::Pipeline>()
            .expect("not a pipeline");
        let webrtcbin = pipeline
            .get_by_name("webrtcbin")
            .expect("can't find webrtcbin");

        webrtcbin.set_property_from_str("stun-server", constants::STUN_SERVER);
        webrtcbin.set_property_from_str("turn-server", constants::TURN_SERVER);
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        webrtcbin
            .emit(
                "add-transceiver",
                &[
                    &gstreamer_webrtc::WebRTCRTPTransceiverDirection::Recvonly,
                    &gstreamer::Caps::new_simple(
                        "application/x-rtp",
                        &[
                            ("media", &"audio"),
                            ("encoding-name", &"OPUS"),
                            ("payload", &(97i32)),
                            ("clock-rate", &(48_000i32)),
                            ("encoding-params", &"2"),
                        ],
                    ),
                ],
            )
            .unwrap();

        let bus = pipeline.get_bus().unwrap();
        let bus_stream = bus.stream();

        let room_name = room_name.clone();
        let uuid = uuid.clone();
        let room_address = room_address.clone();
        let app = App(Arc::new(AppInner {
            pipeline,
            webrtcbin,
            room_address,
            room_name,
            uuid,
        }));
        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin
            .connect("on-negotiation-needed", false, move |_values| {
                let app = upgrade_weak_reference!(app_clone, None);
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

                let app = upgrade_weak_reference!(app_clone, None);
                app.on_ice_candidate(&candidate, &sdp_mline_index);
                None
            })
            .unwrap();

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

        Ok((app, bus_stream))
    }

    fn downgrade_to_weak_reference(&self) -> AppWeak {
        AppWeak(Arc::downgrade(&self.0))
    }

    fn on_negotiation_needed(&self) {
        info!("Starting Negotiation");
        let app_clone = self.downgrade_to_weak_reference();
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            let app = upgrade_weak_reference!(app_clone);
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
                    message_websocket::MessageSocketType::SignallingOfferSDP {
                        value: offer.get_sdp().as_text().unwrap(),
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

impl Drop for AppInner {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}

pub struct WebRTC {
    app: App,
}

impl WebRTC {
    pub fn new(
        room_address: &Addr<service_room::Room>,
        room_name: &String,
        uuid: &String,
    ) -> Addr<WebRTC> {
        let (app, _bus_stream) = App::new(&room_address, &room_name, &uuid).unwrap();
        let webrtc = WebRTC { app: app };
        webrtc.start()
    }
}

impl Actor for WebRTC {
    type Context = Context<Self>;
}

impl Handler<webrtc::SessionDescription> for WebRTC {
    type Result = ();

    fn handle(&mut self, _channel: webrtc::SessionDescription, _: &mut Context<Self>) {
        info!("OK");
    }
}

impl Handler<webrtc::CheckRunning> for WebRTC {
    type Result = ();

    fn handle(&mut self, _channel: webrtc::CheckRunning, _: &mut Context<Self>) {
        info!("STILL RUNNING");
    }
}

impl Handler<webrtc::ICECandidate> for WebRTC {
    type Result = ();

    fn handle(&mut self, channel: webrtc::ICECandidate, _: &mut Context<Self>) {
        info!("EMIT");
        self.app
            .webrtcbin
            .emit(
                "add-ice-candidate",
                &[&channel.sdp_mline_index, &channel.candidate],
            )
            .unwrap();
    }
}

impl Handler<webrtc::SDPAnswer> for WebRTC {
    type Result = ();

    fn handle(&mut self, answer: webrtc::SDPAnswer, _: &mut Context<Self>) {
        let sdp = serde_json::to_string(&answer).unwrap();
        let ret = gstreamer_sdp::SDPMessage::parse_buffer(sdp.as_bytes())
            .map_err(|_| info!("Failed to parse SDP answer"))
            .unwrap();
        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(
            gstreamer_webrtc::WebRTCSDPType::Answer,
            ret,
        );

        let promise = gstreamer::Promise::with_change_func(move |_reply| {
            info!("DONESSSSSSSSSSSs");
        });

        self.app
            .webrtcbin
            .emit("set-remote-description", &[&answer, &promise])
            .unwrap();
    }
}
