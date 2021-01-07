use actix_derive::Message;
use actix::Addr;
use crate::service::room;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterUser {
    pub uuid: String,
    pub room_name: String,
    pub room_address: Addr<room::Room>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct DeleteUser {
    pub uuid: String,
    pub room_name: String,
}