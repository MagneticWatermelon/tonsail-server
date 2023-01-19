use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::{HeaderMap, StatusCode};
use tracing::instrument;
// use tracing::error;

use crate::prisma::organization;

use super::AppState;

#[instrument(name = "Fetching organization", skip_all)]
pub async fn get_organization(
    Path(org_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // This can not dail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .organization()
        .find_first(vec![organization::id::equals(org_id)])
        .with(organization::users::fetch(vec![]))
        .exec()
        .await
        .unwrap();

    match resp {
        Some(data) => Json(data).into_response(),
        None => (StatusCode::NOT_FOUND, "No such organization exists").into_response(),
    }
}
