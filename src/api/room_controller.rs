use crate::constants;
use crate::models::room as models_room;
use crate::service::room as service_room;
use crate::models::supervisor as supervisor;
use crate::service::{session, webrtc};
use crate::models::{
    response,
};

use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub async fn create(
    request: web::Json<models_room::CreateRoom>,
    room_address: web::Data<Addr<service_room::Room>>,
) -> Result<HttpResponse, Error> {
    room_address.get_ref().do_send(models_room::CreateRoom {
        name: request.name.to_owned(),
        master_uuid: request.master_uuid.to_owned(),
    });
    Ok(HttpResponse::Ok().json(
        response::ResponseBody::new(
            constants::MESSAGE_OK, 
            constants::MESSAGE_ROOM_CREATED
        )
    ))
}

pub async fn get_all_room(
    room_address: web::Data<Addr<service_room::Room>>
) -> Result<HttpResponse, Error> {
    let rooms = room_address.get_ref().send(
        models_room::GetListRoom {}
    ).await;

    Ok(HttpResponse::Ok().json(
        response::ResponseBody::new(
            constants::MESSAGE_OK, rooms.unwrap()
        )
    ))
}

pub async fn join(
    parameter: web::Path<(String, String)>,
    request: HttpRequest,
    stream: web::Payload,
    supervisor_webrtc_address: web::Data<Addr<webrtc::supervisor::Supervisor>>,
    room_address: web::Data<Addr<service_room::Room>>,
) -> Result<HttpResponse, Error> {
    let master_uuid = room_address.get_ref().send(models_room::GetMaster {
        room_name: parameter.0.0.clone()
    } ).await.unwrap();

    if &master_uuid != "NAN" {
        supervisor_webrtc_address.get_ref().send(supervisor::RegisterUser {
            room_address: room_address.get_ref().clone(),
            room_name: parameter.0.0.clone(),
            uuid: parameter.0.1.to_owned()
        } ).await.unwrap();

        let response = ws::start(
            session::Session {
                room_name: parameter.0.0.to_owned(),
                uuid: parameter.0.1.to_owned(),
                room_address: room_address.get_ref().clone(),
                master_uuid: master_uuid,
                webrtc_supervisor_address: supervisor_webrtc_address.get_ref().clone(),
            },
            &request,
            stream,
        );

        response
    } else {
        Ok(HttpResponse::Forbidden().json(
            response::ResponseBody::new(
                constants::MESSAGE_ERROR, 
                constants::MESSAGE_ROOM_DOESNT_EXIST
            )
        ))
    }

}

pub async fn delete_room(
    request: web::Json<models_room::DeleteRoom>,
    room_address: web::Data<Addr<service_room::Room>>,
) -> Result<HttpResponse, Error> {
    let _ = room_address
        .get_ref()
        .do_send(models_room::DeleteRoom {
            name: request.name.to_owned(),
        });

    Ok(HttpResponse::Ok().json(
        response::ResponseBody::new(
            constants::MESSAGE_OK,
            constants::MESSAGE_ROOM_DELETED,
        )
    ))
}

pub async fn kick_user(
    request: web::Json<supervisor::DeleteUser>,
    supervisor_webrtc_address: web::Data<Addr<webrtc::supervisor::Supervisor>>,
    _room_address: web::Data<Addr<service_room::Room>>,
) -> Result<HttpResponse, Error> {

    let _ = supervisor_webrtc_address
        .get_ref()
        .do_send(supervisor::DeleteUser {
            uuid: request.uuid.to_owned(),
            room_name: request.room_name.to_owned(),
        });

    // let _ = room_address
    //     .get_ref()
    //     .do_send(models_room::KickUser {
    //         uuid: request.uuid.to_owned(),
    //         room_name: request.room_name.to_owned(),
    //     });

    Ok(HttpResponse::Ok().json(response::ResponseBody::new(
        constants::MESSAGE_OK,
        constants::MESSAGE_USER_KICKED,
    )))
}
