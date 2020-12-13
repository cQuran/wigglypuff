use crate::{
    constants,
    models::network_transversal::{NetworkTransversal, Stun, Turn},
    models::response::ResponseBody,
};
use actix_web::{HttpResponse, Result};

pub async fn get_stun_address() -> Result<HttpResponse> {
    let stun_address = Stun {
        address: "stun:global.stun.twilio.com:3478?transport=udp".to_string(),
    };
    let mut turns = Vec::new();
    turns.push(Turn {
        address: "turn:global.turn.twilio.com:3478?transport=udp".to_string(),
        username: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
        credential: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
    });
    turns.push(Turn {
        address: "turn:global.turn.twilio.com:3478?transport=tcp".to_string(),
        username: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
        credential: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
    });
    turns.push(Turn {
        address: "turn:global.turn.twilio.com:443?transport=tcp".to_string(),
        username: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
        credential: "e994ab564e859690d6e325ae7b2e08b0b42ac468836921fc7db5ebb2d080958c".to_string(),
    });
    let network_transversal = NetworkTransversal {stun: stun_address, turn: turns};
    Ok(HttpResponse::Ok().json(ResponseBody::new(
        constants::MESSAGE_OK,
        network_transversal,
    )))
}
