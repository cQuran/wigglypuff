use crate::api::network_transversal_controller;
use actix_web::web;
use log::info;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    info!("Configurating routes...");
    cfg.service(
        web::scope("/api")
            .service(web::scope("/network_transversal").service(
                web::resource("").route(web::get().to(network_transversal_controller::get_stun_address)),
            )),
    );
}
