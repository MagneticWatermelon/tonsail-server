use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct HttpError {
    #[serde(skip_serializing)]
    pub status_code: StatusCode,
    pub body: HttpErrorBody,
}

#[derive(Serialize)]
pub struct HttpErrorBody {
    pub title: String,
    pub message: String,
}

impl HttpError {
    pub fn new(status_code: StatusCode, title: &str, message: &str) -> Self {
        Self {
            status_code,
            body: HttpErrorBody {
                title: title.to_string(),
                message: message.to_string(),
            },
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}
