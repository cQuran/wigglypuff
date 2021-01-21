use crate::{
    constants,
    models::{firebase_cloud_message, response},
};
use actix_web::{HttpResponse, Result};

pub async fn get_firebase_cloud_message_token() -> Result<HttpResponse> {
    let token_fcm = firebase_cloud_message::FCM {
        token: &constants::FCM_API_TOKEN_KEY,
    };

    Ok(HttpResponse::Ok().json(response::ResponseBody::FcmTokens(token_fcm)))
}
