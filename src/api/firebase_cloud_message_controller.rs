use crate::{constants, models::firebase_cloud_message::FCM, models::response::ResponseBody};
use actix_web::{HttpResponse, Result};

pub async fn get_firebase_cloud_message_token() -> Result<HttpResponse> {
    let token_fcm = FCM {
        token: constants::FCM_API_TOKEN_KEY.to_owned(),
    };

    Ok(HttpResponse::Ok().json(ResponseBody::new(constants::MESSAGE_OK, token_fcm)))
}
