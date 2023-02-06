use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{extract::State, Form};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::instrument;

use crate::prisma::{project, test};
use crate::util::nano_id::generate_id;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateForm {
    name: String,
    project_id: String,
}

#[instrument(name = "Creating new test", skip_all)]
pub async fn create_test(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(test): Form<CreateForm>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .test()
        .create(
            generate_id(),
            test.name,
            project::id::equals(test.project_id),
            vec![],
        )
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[instrument(name = "Fetching test", skip_all)]
pub async fn get_test(
    Path(test_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .test()
        .find_first(vec![test::id::equals(test_id)])
        .with(test::runs::fetch(vec![]))
        .exec()
        .await
        .unwrap();

    match resp {
        Some(data) => Json(data).into_response(),
        None => (StatusCode::NOT_FOUND, "No such test exists").into_response(),
    }
}
