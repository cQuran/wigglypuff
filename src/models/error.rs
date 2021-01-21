use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WigglypuffError {
    #[error("requested file was not found")]
    NotFound,
    #[error("you are forbidden to access requested file.")]
    Forbidden,
    #[error("unknown Internal Error")]
    Unknown,
    #[error("unknown Internal Error")]
    MailboxError,
    #[error("room already exist!")]
    RoomAlreadyExist,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl std::convert::From<actix::MailboxError> for WigglypuffError {
    fn from(_: actix::MailboxError) -> Self {
        WigglypuffError::MailboxError
    }
}

impl ResponseError for WigglypuffError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MailboxError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RoomAlreadyExist => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}
