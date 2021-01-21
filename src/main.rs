use actix_web::{App, HttpServer};

mod api;
mod config;
mod constants;
mod models;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = config::input_arguments::config_arguments();
    let room = service::room::Room::new();
    let webrtc_supervisor = service::webrtc::supervisor::Supervisor::new();

    config::gstreamer::check_plugins();

    HttpServer::new(move || {
        App::new()
            .data(room.clone())
            .data(webrtc_supervisor.clone())
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind(&url)?
    .run()
    .await
}
