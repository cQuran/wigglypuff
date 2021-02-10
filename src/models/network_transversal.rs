use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum STUNTURN {
    TURN {
        url: String,
        urls: String,
        username: String,
        credential: String,
    },
    STUN {
        url: String,
        urls: String,
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Twilio {
    pub username: String,
    pub ice_servers: Vec<STUNTURN>,
    pub date_updated: String,
    pub account_sid: String,
    pub ttl: String,
    pub date_created: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkTransversal(pub Vec<STUNTURN>);
