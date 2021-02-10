use actix_web::{App, HttpServer};
use std::sync::{Arc, Mutex};

mod api;
mod config;
mod constants;
mod models;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = config::input_arguments::config_arguments();
    let room = service::room::Room::new();
    config::gstreamer::check_plugins();

    let _sentry_guard = sentry::init((constants::SENTRY_TOKEN, sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    let nat = Arc::new(Mutex::new(Vec::<models::network_transversal::STUNTURN>::new()));
    let webrtc_supervisor = service::webrtc::supervisor::Supervisor::new(nat.clone());
    HttpServer::new(move || {
        App::new()
            .data(room.clone())
            .data(nat.clone())
            .data(webrtc_supervisor.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(sentry_actix::Sentry::new())
            .configure(config::app::config_services)
    })
    .bind(&url)?
    .run()
    .await
}
