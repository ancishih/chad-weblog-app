use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    Error as AxumError,
};
use cron::error::Error as CronError;
use lettre::error::Error as LettreError;
use serde_json::json;
use std::env::VarError;
pub type Result<T> = core::result::Result<T, Error>;

// pub struct Error;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("this error message came from sqlx: {0}")]
    SystemSqlxError(#[from] sqlx::Error),

    #[error("{0}")]
    SystemReqwestError(#[from] reqwest::Error),

    #[error("Load Envirnoment Variable Error:{0}")]
    SystemLoadParametersError(#[from] VarError),

    #[error("{0}")]
    LettreFail(#[from] LettreError),

    #[error("{0}")]
    SystemAxumError(#[from] AxumError),

    #[error("{0}")]
    SystemCronError(#[from] CronError),

    #[error("{0} is Not Found.")]
    NOTFOUND(String),

    #[error("{0}")]
    BADREQUEST(String),

    #[error("{0}")]
    CUSTOMERROR(String),
}

impl Error {
    fn get_code(&self) -> (StatusCode, u16) {
        match *self {
            Error::BADREQUEST(_) => (StatusCode::BAD_REQUEST, 400),
            Error::NOTFOUND(_) => (StatusCode::NOT_FOUND, 404),
            // Error::SectError(_) => (StatusCode::NOT_FOUND, 404),
            Error::LettreFail(_) => (StatusCode::BAD_GATEWAY, 502),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, 500),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_code();
        let message = self.to_string();
        let body = Json(json!({ "status_code": code, "message":message }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthenticateError {
    #[error("Invalid authentication credentials")]
    InvalidToken,
}
