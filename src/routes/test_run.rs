use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{extract::State, Form};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::instrument;

use crate::prisma::{test, test_run};
use crate::util::nano_id::generate_id;

use super::AppState;

#[derive(Deserialize)]
pub struct CreateForm {
    test_id: String,
}

#[instrument(name = "Creating new test run", skip_all)]
pub async fn create_test_run(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(test_run): Form<CreateForm>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .test_run()
        .create(generate_id(), test::id::equals(test_run.test_id), vec![])
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[instrument(name = "Fetching test run", skip_all)]
pub async fn get_test_run(
    Path(run_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .test_run()
        .find_first(vec![test_run::id::equals(run_id)])
        .exec()
        .await
        .unwrap();

    match resp {
        Some(data) => Json(data).into_response(),
        None => (StatusCode::NOT_FOUND, "No such run exists").into_response(),
    }
}
