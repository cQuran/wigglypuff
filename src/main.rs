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
    let config_https = config::https::config_https();

    config::webrtc::config_gstreamer();

    HttpServer::new(move || {
        App::new()
            .data(room.clone())
            .data(webrtc_supervisor.clone())
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind_openssl(&url, config_https)?
    .run()
    .await
}
