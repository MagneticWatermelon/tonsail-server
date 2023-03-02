use super::http_error::HttpError;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use prisma_client_rust::QueryError;

/// Our app's top level error type.
pub enum AppError {
    Http(HttpError),
    Db(QueryError),
}

/// This makes it possible to use `?` to automatically convert a `HttpError`
/// into an `AppError`.
impl From<HttpError> for AppError {
    fn from(inner: HttpError) -> Self {
        AppError::Http(inner)
    }
}

impl From<QueryError> for AppError {
    fn from(inner: QueryError) -> Self {
        AppError::Db(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Http(e) => e.into_response(),
            AppError::Db(QueryError::Execute(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_owned()).into_response()
            }
            AppError::Db(QueryError::Serialize(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            AppError::Db(QueryError::Deserialize(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
