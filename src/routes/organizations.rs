use super::AppState;
use crate::prisma::organization;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form, Json,
};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::instrument;

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

#[derive(Deserialize)]
pub struct UpdateForm {
    name: String,
}

#[instrument(name = "Updating organization", skip_all)]
pub async fn update_organization(
    Path(org_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(org): Form<UpdateForm>,
) -> Response {
    // This can not dail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .organization()
        .update(
            organization::id::equals(org_id),
            vec![organization::name::set(org.name)],
        )
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "No such organization exists").into_response(),
    }
}
