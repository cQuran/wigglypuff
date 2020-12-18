use crate::constants;
use crate::models::{
    response::ResponseBody,
    room::{CreateRoom, DeleteRoom, GetListRoom, GetMaster, Room, KickUser},
    session::Session,
};
use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub async fn create(
    request: web::Json<CreateRoom>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    address.get_ref().do_send(CreateRoom {
        name: request.name.to_owned(),
        master_uuid: request.master_uuid.to_owned(),
    });

    Ok(HttpResponse::Ok().json(ResponseBody::new(constants::MESSAGE_OK, constants::MESSAGE_ROOM_CREATED)))
}

pub async fn list_room(address: web::Data<Addr<Room>>) -> Result<HttpResponse, Error> {
    let data = address.get_ref().send(GetListRoom {}).await;

    Ok(HttpResponse::Ok().json(ResponseBody::new(constants::MESSAGE_OK, data.unwrap())))
}

pub async fn join(
    parameter: web::Path<(String, String)>,
    request: HttpRequest,
    stream: web::Payload,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let master_uuid = address.get_ref().send(GetMaster {room_name: parameter.0.0.clone()}).await.unwrap();

    if &master_uuid != "NAN" {
        let response = ws::start(
            Session {
                room_name: parameter.0.0.to_owned(),
                uuid: parameter.0.1.to_owned(),
                address: address.get_ref().clone(),
                master_uuid: master_uuid
            },
            &request,
            stream,
        );

        response
    } else {
        Ok(HttpResponse::Forbidden().json(ResponseBody::new(constants::MESSAGE_ERROR, constants::MESSAGE_ROOM_DOESNT_EXIST)))
    }

}

pub async fn delete_room(
    request: web::Json<DeleteRoom>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let _ = address
        .get_ref()
        .do_send(DeleteRoom {
            name: request.name.to_owned(),
        });

    Ok(HttpResponse::Ok().json(ResponseBody::new(
        constants::MESSAGE_OK,
        constants::MESSAGE_ROOM_DELETED,
    )))
}

pub async fn kick_user(
    request: web::Json<KickUser>,
    address: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let _ = address
        .get_ref()
        .do_send(KickUser {
            uuid: request.uuid.to_owned(),
            room_name: request.room_name.to_owned(),
        });

    Ok(HttpResponse::Ok().json(ResponseBody::new(
        constants::MESSAGE_OK,
        constants::MESSAGE_USER_KICKED,
    )))
}
