use crate::constants;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Running, StreamHandler};
use actix_derive::{Message, MessageResponse};
use anyhow::Error;
use futures::{Stream, StreamExt};
use gstreamer;
use gstreamer::{
    prelude::{Cast, ObjectExt},
    ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstBinExtManual,
};
use serde::{Deserialize, Serialize};
use glib;

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

#[derive(Debug, Clone)]
struct App(Arc<AppInner>);

#[derive(Debug, Clone)]
struct AppWeak(Weak<AppInner>);

#[derive(Debug)]
struct AppInner {
    pipeline: gstreamer::Pipeline,
    webrtcbin: gstreamer::Element,
}

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
    fn new() -> Result<(Self, impl Stream<Item = gstreamer::Message>), Error> {
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

        let bus = pipeline.get_bus().unwrap();
        let bus_stream = bus.stream();

        let app = App(Arc::new(AppInner {
            pipeline,
            webrtcbin,
        }));

        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin
            .connect("on-negotiation-needed", false, move |values| {
                let _webrtc = values[0].get::<gstreamer::Element>().unwrap();
                let app = upgrade_weak_reference!(app_clone, None);
                info!("START RUNNING WEBRTC");
                None
            })
            .unwrap();

        let app_clone = app.downgrade_to_weak_reference();
        app.webrtcbin
            .connect("on-ice-candidate", false, move |values| {
                let _webrtc = values[0]
                    .get::<gstreamer::Element>()
                    .expect("Invalid argument");
                let _mlineindex = values[1].get_some::<u32>().expect("Invalid argument");
                let _candidate = values[2]
                    .get::<String>()
                    .expect("Invalid argument")
                    .unwrap();

                let _app = upgrade_weak_reference!(app_clone, None);

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
    fn handle_negotiation_needed() {}

    fn handle_ice_candidate(mlineindex: &String, candidate: &String) {}

    fn handle_sdp(&self, types: &str, sdp: &str) -> Result<(), Error> {
        if types == "answer" {
            info!("Received answer:\n{}\n", sdp);
        } else if types == "offer" {
            info!("Received offer:\n{}\n", sdp);
        } else {
            info!("Sdp type is not \"answer\" but \"{}\"", types);
        }
        Ok(())
    }

    fn downgrade_to_weak_reference(&self) -> AppWeak {
        AppWeak(Arc::downgrade(&self.0))
    }
}

impl Drop for AppInner {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}

pub struct WebRTC {}

impl WebRTC {
    pub fn new() -> Addr<WebRTC> {
        let webrtc = WebRTC {};
        webrtc.start()
    }
}

impl Actor for WebRTC {
    type Context = Context<Self>;
}

#[derive(Message, Deserialize)]
#[rtype(result = "()")]
pub struct CreateWebRTCChannel {
    pub room_name: String,
}

impl Handler<CreateWebRTCChannel> for WebRTC {
    type Result = ();

    fn handle(&mut self, channel: CreateWebRTCChannel, _: &mut Context<Self>) {
        info!("START RUNNING");
        let (app, bus_stream) = App::new().unwrap();
        let main_loop = glib::MainLoop::new(None, false);
        let mut bus_stream = bus_stream.fuse();
        main_loop.run();
    }
}
