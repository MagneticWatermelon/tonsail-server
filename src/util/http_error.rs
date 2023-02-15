use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct HttpError {
    #[serde(skip_serializing)]
    status_code: StatusCode,
    title: String,
    message: String,
}

impl HttpError {
    pub fn new(status_code: StatusCode, title: &str, message: &str) -> Self {
        Self {
            status_code,
            title: title.to_string(),
            message: message.to_string(),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}
