use axum::body;
use axum::extract::rejection::{FormRejection, QueryRejection};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use prisma_client_rust::QueryError;
use thiserror::Error;

/// Our app's top level error type.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Not authorized: {0}")]
    UnAuthorized(String),

    #[error("Require Admin privileges for {0}")]
    RequireAdmin(String),

    #[error(transparent)]
    Db(QueryError),

    #[error(transparent)]
    PostgresError(#[from] sqlx::Error),

    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),

    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
}

impl From<QueryError> for AppError {
    fn from(inner: QueryError) -> Self {
        AppError::Db(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::UnAuthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::RequireAdmin(_) => StatusCode::FORBIDDEN,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::AxumFormRejection(_) => StatusCode::BAD_REQUEST,
            AppError::AxumQueryRejection(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let e = &self;
        let body = body::boxed(body::Full::from(e.to_string()));
        tracing::error!(error = e.to_string());
        axum::response::Response::builder()
            .header("Content-Type", "text/plain")
            .status(status)
            .body(body)
            .unwrap()
    }
}
