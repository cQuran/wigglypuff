use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Stun {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct Turn {
    pub address: String,
    pub username: String,
    pub credential: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkTransversal {
    pub stun: Stun,
    pub turn: Vec<Turn>,
}
