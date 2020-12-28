use crate::api::{
    firebase_cloud_message_controller, network_transversal_controller, room_controller,
};
use actix_web::{guard, web};
use log::info;

pub fn config_services(config: &mut web::ServiceConfig) {
    info!("Configurating routes...");
    config.service(
        web::scope("").service(
            web::scope("/api")
                .service(
                    web::scope("/network_transversal")
                        .service(web::resource("").route(
                            web::get().to(network_transversal_controller::get_stun_address),
                        )),
                )
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
                    web::scope("/info").service(web::resource("").route(
                        web::get().to(
                            firebase_cloud_message_controller::get_firebase_cloud_message_token,
                        ),
                    )),
                ))
                .service(
                    web::scope("/room")
                        .service(
                            web::scope("/create").service(
                                web::resource("").route(
                                    web::post()
                                        .guard(guard::Header("content-type", "application/json"))
                                        .to(room_controller::create),
                                ),
                            ),
                        )
                        .service(
                            web::scope("/join/{room_name}/{id}").service(
                                web::resource("").route(web::get().to(room_controller::join)),
                            ),
                        )
                        .service(
                            web::scope("/delete").service(
                                web::resource("").route(
                                    web::post()
                                        .guard(guard::Header("content-type", "application/json"))
                                        .to(room_controller::delete_room),
                                ),
                            ),
                        )
                        .service(web::scope("/info").service(
                            web::resource("").route(web::get().to(room_controller::list_room)),
                        )),
                ),
        ),
    );
}
