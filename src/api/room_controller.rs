use crate::constants;
use crate::models::{
    response::ResponseBody,
    room::{CreateRoom, CreateRoomWithKey, DeleteRoom, GetListRoom, Room},
    session::{Key, Session},
};
use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rand::{distributions::Alphanumeric, Rng};

pub async fn create(
    request: web::Json<CreateRoom>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let key = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(10)
        .collect::<String>();

    address.get_ref().do_send(CreateRoomWithKey {
        key: key.clone(),
        name: request.name.to_owned(),
        master_uuid: request.master_uuid.to_owned(),
    });

    Ok(HttpResponse::Ok().json(ResponseBody::new(constants::MESSAGE_OK, Key { key: key })))
}

pub async fn list_room(address: web::Data<Addr<Room>>) -> Result<HttpResponse, Error> {
    let data = address.get_ref().send(GetListRoom {}).await;

    Ok(HttpResponse::Ok().json(ResponseBody::new(constants::MESSAGE_OK, data.unwrap())))
}

pub async fn join(
    name: web::Path<String>,
    request: HttpRequest,
    stream: web::Payload,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let response = ws::start(
        Session {
            name: name.to_owned(),
            address: address.get_ref().clone(),
        },
        &request,
        stream,
    );
    response
}

pub async fn delete(
    request: web::Json<DeleteRoom>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let data = address
        .get_ref()
        .send(DeleteRoom {
            name: request.name.to_owned(),
        })
        .await;

    Ok(HttpResponse::Ok().json(ResponseBody::new(
        constants::MESSAGE_OK,
        constants::MESSAGE_DELETED,
    )))
}

pub async fn broadcast(
    request: web::Json<CreateRoom>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(ResponseBody::new(
        constants::MESSAGE_OK,
        constants::MESSAGE_OK,
    )))
}
