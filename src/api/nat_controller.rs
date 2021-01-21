use crate::{
    constants,
    models::{network_transversal, response},
};
use actix_web::{HttpResponse, Result};

pub async fn get_stun_address() -> Result<HttpResponse> {
    let stun_address = network_transversal::Stun {
        address: constants::STUN_SERVER.to_string(),
    };
    let mut turns = Vec::new();
    for turn in constants::TURN_ADDRESS.iter() {
        turns.push(network_transversal::Turn {
            address: turn.to_string(),
            username: constants::TURN_USERNAME.to_string(),
            credential: constants::TURN_CREDENTIAL.to_string(),
        });
    }
    let network_transversal = network_transversal::NetworkTransversal {
        stun: stun_address,
        turn: turns,
    };
    Ok(HttpResponse::Ok().json(response::ResponseBody::Addresses(network_transversal)))
}
