use crate::api::{
    firebase_cloud_message_controller, network_transversal_controller, room_controller,
};
use actix_web::{guard, web, error, HttpResponse};
use log::info;

pub fn config_services(config: &mut web::ServiceConfig) {
    info!("[CONFIGURATING ACTOR ROUTE]");
    config
        .service(
            web::scope("").service(
                web::scope("/api")
                    .service(
                        web::scope("/network_transversal").service(web::resource("").route(
                            web::get().to(network_transversal_controller::get_stun_address),
                        )),
                    )
                    .service(web::scope("/webrtc.js").service(
                        web::resource("").route(web::get().to(room_controller::get_webrtc_js)),
                    ))
                    .service(web::scope("/webrtc").service(
                        web::resource("").route(web::get().to(room_controller::get_webrtc_client)),
                    ))
                    .service(
                        web::scope("/user/kick").service(
                            web::resource("").route(
                                web::post()
                                    .guard(guard::Header("content-type", "application/json"))
                                    .to(room_controller::kick_user),
                            ),
                        ),
                    )
                    .service(web::scope("/firebase_cloud_message").service(
                        web::scope("/info").service(web::resource("").route(web::get().to(
                            firebase_cloud_message_controller::get_firebase_cloud_message_token,
                        ))),
                    ))
                    .service(
                        web::scope("/room")
                            .service(
                                web::scope("/create").service(
                                    web::resource("").route(
                                        web::post()
                                            .guard(guard::Header(
                                                "content-type",
                                                "application/json",
                                            ))
                                            .to(room_controller::create),
                                    ),
                                ),
                            )
                            .service(web::scope("/join/{room_name}/{uuid}").service(
                                web::resource("").route(web::get().to(room_controller::join)),
                            ))
                            .service(
                                web::scope("/delete").service(
                                    web::resource("").route(
                                        web::post()
                                            .guard(guard::Header(
                                                "content-type",
                                                "application/json",
                                            ))
                                            .to(room_controller::delete_room),
                                    ),
                                ),
                            )
                            .service(
                                web::scope("/info").service(
                                    web::resource("")
                                        .route(web::get().to(room_controller::get_all_room)),
                                ),
                            ),
                    ),
            ),
        )
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            error::InternalError::from_response(
                "",
                HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(format!(r#"{{"error":"{}"}}"#, err)),
            )
            .into()
        }));
}
