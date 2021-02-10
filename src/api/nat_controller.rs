use crate::constants;
use crate::models::{network_transversal, response};

use actix_web::{web, HttpResponse, Result};
use std::env;
use std::mem;
use std::sync::{Arc, Mutex};

pub async fn get_stun_address(
    nat: web::Data<Arc<Mutex<Vec<network_transversal::STUNTURN>>>>,
) -> Result<HttpResponse> {
    let nat_address = nat.lock().unwrap();

    let results: Vec<network_transversal::STUNTURN> = nat_address
        .iter()
        .map(|room_name| room_name.clone())
        .collect();

    let network_transversal = network_transversal::NetworkTransversal(results);

    Ok(HttpResponse::Ok().json(response::ResponseBody::IceServers(network_transversal)))
}

pub async fn refresh_twilio(
    nat: web::Data<Arc<Mutex<Vec<network_transversal::STUNTURN>>>>,
) -> Result<HttpResponse> {
    let sid = env::var("TWILIO_SID").unwrap();
    let token = env::var("TWILIO_TOKEN").unwrap();
    let address = format!(
        "https://{sid}:{token}@api.twilio.com/2010-04-01/Accounts/{sid}/Tokens.json",
        sid = sid,
        token = token
    );
    let client = reqwest::blocking::Client::new();
    let resp: network_transversal::Twilio = client.post(&address).send().unwrap().json().unwrap();
    let mut nat_addresses = nat.lock().unwrap();

    mem::replace(&mut *nat_addresses, resp.ice_servers.clone());
    Ok(HttpResponse::Ok().json(response::ResponseBody::Message(
        constants::MESSAGE_NAT_REFRESHED,
    )))
}
