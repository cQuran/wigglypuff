use crate::{
    constants,
    models::{network_transversal, response},
};
use actix_web::{HttpResponse, Result};

pub async fn get_stun_address() -> Result<HttpResponse> {
    let stun_address = network_transversal::Stun {
        address: constants::STUN_SERVER.to_string(),
    };
    let turn_address = network_transversal::Turn {
        address: constants::TURN_SERVER.to_string(),
    };
    let network_transversal = network_transversal::NetworkTransversal {
        stun: stun_address,
        turn: turn_address,
    };
    Ok(HttpResponse::Ok().json(response::ResponseBody::Addresses(network_transversal)))
}
