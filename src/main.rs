use actix_web::{App, HttpServer};

mod api;
mod config;
mod constants;
mod models;
mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url = config::input_arguments::config_arguments();
    let room = models::room::Room::new();
    config::webrtc::config_gstreamer();
    let webrtc = models::webrtc::WebRTC::new();

    // Simple case create webrtc connection (wip), will be deleted soon after full pipeline works
    webrtc.do_send(models::webrtc::CreateWebRTCChannel {
        room_name: "Test-Channel".to_string(),
    });

    HttpServer::new(move || {
        App::new()
            .data(room.clone())
            .data(webrtc.clone())
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind(&url)?
    .run()
    .await
}
