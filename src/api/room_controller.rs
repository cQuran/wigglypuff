use crate::constants;
use crate::models::{error, response, room as room_models, supervisor};
use crate::service::{room as room_service, session, webrtc};

use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub async fn create(
    request: web::Json<room_models::CreateRoom>,
    room_address: web::Data<Addr<room_service::Room>>,
) -> Result<HttpResponse, error::WigglypuffError> {
    let is_unique = room_address
        .get_ref()
        .send(room_models::CreateRoom {
            name: request.name.to_owned(),
            master_uuid: request.master_uuid.to_owned(),
        })
        .await?;

    if is_unique {
        Ok(HttpResponse::Ok().json(response::ResponseBody::Message(
            constants::MESSAGE_ROOM_CREATED,
        )))
    } else {
        Err(error::WigglypuffError::RoomAlreadyExist)
    }
}

pub async fn get_rooms(
    room_address: web::Data<Addr<room_service::Room>>,
) -> Result<HttpResponse, error::WigglypuffError> {
    let rooms = room_address
        .get_ref()
        .send(room_models::GetRooms {})
        .await?;

    Ok(HttpResponse::Ok().json(response::ResponseBody::Rooms(rooms)))
}

pub async fn join(
    room: web::Path<room_models::Join>,
    request: HttpRequest,
    stream: web::Payload,
    webrtc_address: web::Data<Addr<webrtc::supervisor::Supervisor>>,
    room_address: web::Data<Addr<room_service::Room>>,
) -> Result<HttpResponse, Error> {
    let master_uuid = room_address
        .get_ref()
        .send(room_models::GetMaster {
            room_name: room.room_name.clone(),
        })
        .await
        .unwrap();

    if &master_uuid != "NAN" {
        let response = ws::start(
            session::Session {
                room_name: room.room_name.to_owned(),
                uuid: room.uuid.to_owned(),
                room_address: room_address.get_ref().clone(),
                master_uuid: master_uuid,
                webrtc_address: webrtc_address.get_ref().clone(),
            },
            &request,
            stream,
        );

        if response.is_ok() {
            webrtc_address
                .get_ref()
                .send(supervisor::RegisterUser {
                    room_address: room_address.get_ref().clone(),
                    room_name: room.room_name.clone(),
                    uuid: room.uuid.to_owned(),
                })
                .await
                .unwrap();

            response
        } else {
            Ok(
                HttpResponse::Forbidden().json(response::ResponseBody::Message(
                    constants::MESSAGE_USER_NOT_WEBSOCKET,
                )),
            )
        }
    } else {
        Ok(
            HttpResponse::Forbidden().json(response::ResponseBody::Message(
                constants::MESSAGE_ROOM_DOESNT_EXIST,
            )),
        )
    }
}

pub async fn delete_room(
    room: web::Json<room_models::DeleteRoom>,
    room_address: web::Data<Addr<room_service::Room>>,
) -> Result<HttpResponse, error::WigglypuffError> {
    room_address
        .get_ref()
        .send(room_models::DeleteRoom {
            name: room.name.to_owned(),
        })
        .await?;

    Ok(HttpResponse::Ok().json(response::ResponseBody::Message(
        constants::MESSAGE_ROOM_DELETED,
    )))
}

pub async fn delete_user(
    user: web::Json<supervisor::DeleteUser>,
    webrtc_address: web::Data<Addr<webrtc::supervisor::Supervisor>>,
    room_address: web::Data<Addr<room_service::Room>>,
) -> Result<HttpResponse, error::WigglypuffError> {
    webrtc_address
        .get_ref()
        .send(supervisor::DeleteUser {
            uuid: user.uuid.to_owned(),
            room_name: user.room_name.to_owned(),
        })
        .await?;

    room_address
        .get_ref()
        .send(room_models::KickUser {
            uuid: user.uuid.to_owned(),
            room_name: user.room_name.to_owned(),
        })
        .await?;

    Ok(HttpResponse::Ok().json(response::ResponseBody::Message(
        constants::MESSAGE_USER_KICKED,
    )))
}
