use crate::api::{network_transversal_controller, room_controller};
use actix_web::{guard, web};
use log::info;

pub fn config_services(config: &mut web::ServiceConfig) {
    info!("Configurating routes...");
    config.service(
        web::scope("")
            .service(
                web::scope("/api/network_transversal").service(
                    web::resource("")
                        .route(web::get().to(network_transversal_controller::get_stun_address)),
                ),
            )
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
                        web::scope("/join/{id}")
                            .service(web::resource("").route(web::get().to(room_controller::join))),
                    )
                    .service(
                        web::scope("/delete").service(
                            web::resource("").route(
                                web::post()
                                    .guard(guard::Header("content-type", "application/json"))
                                    .to(room_controller::delete),
                            ),
                        ),
                    )
                    .service(
                        web::scope("/all").service(
                            web::resource("").route(web::get().to(room_controller::list_room)),
                        ),
                    )
                    .service(
                        web::scope("/broadcast").service(
                            web::resource("").route(
                                web::post()
                                    .guard(guard::Header("content-type", "application/json"))
                                    .to(room_controller::broadcast),
                            ),
                        ),
                    ),
            ),
    );
}
