use actix_derive::Message;
use actix::Addr;
use crate::service::room;
use serde::Deserialize;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterUser {
    pub uuid: String,
    pub room_name: String,
    pub room_address: Addr<room::Room>,
}

#[derive(Message, Clone, Deserialize)]
#[rtype(result = "()")]
pub struct DeleteUser {
    pub uuid: String,
    pub to_uuid: String,
    pub room_name: String,
}