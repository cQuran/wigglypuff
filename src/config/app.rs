use crate::api::{fcm_controller, nat_controller, room_controller, static_web_controller};
use crate::constants;

use actix_web::{error, guard, web, HttpResponse};
use log::info;

pub fn config_services(config: &mut web::ServiceConfig) {
    info!("[CONFIGURATING ACTOR ROUTE]");
    config
        .service(
            web::scope("/api")
                .service(
                    web::scope("/info")
                        .service(
                            web::scope("/network_transversal").service(
                                web::resource("")
                                    .route(web::get().to(nat_controller::get_stun_address)),
                            ),
                        )
                        .service(web::scope("/firebase_cloud_message").service(
                            web::resource("").route(
                                web::get().to(fcm_controller::get_firebase_cloud_message_token),
                            ),
                        )),
                )
                .service(
                    web::scope("/user").service(
                        web::resource("").route(
                            web::delete()
                                .guard(guard::Header("content-type", "application/json"))
                                .to(room_controller::delete_user),
                        ),
                    ),
                )
                .service(
                    web::scope("/room")
                        .service(
                            web::resource("")
                                .route(
                                    web::delete()
                                        .guard(guard::Header("content-type", "application/json"))
                                        .to(room_controller::delete_room),
                                )
                                .route(web::get().to(room_controller::get_rooms))
                                .route(
                                    web::put()
                                        .guard(guard::Header("content-type", "application/json"))
                                        .to(room_controller::create),
                                ),
                        ),
                ),
        )
        .service(
            web::scope("websocket/{room_name}/{uuid}").service(
                web::resource("").route(web::get().to(room_controller::join)),
            ),
        )
        .service(
            web::scope("/static")
                .service(web::scope("/webrtc.js").service(
                    web::resource("").route(web::get().to(static_web_controller::get_webrtc_js)),
                ))
                .service(
                    web::scope("/webrtc").service(
                        web::resource("")
                            .route(web::get().to(static_web_controller::get_webrtc_client)),
                    ),
                ),
        )
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            error::InternalError::from_response(
                constants::MESSAGE_JSON_PARSE_ERROR,
                HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!(r#"{{"error":"{}"}}"#, err)),
            )
            .into()
        }));
}
