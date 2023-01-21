use super::AppState;
use crate::{
    prisma::{organization, project},
    util::nano_id::generate_id,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form, Json,
};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize)]
pub struct CreateForm {
    name: String,
    organization_id: String,
}

#[instrument(name = "Creating new project", skip_all)]
pub async fn create_project(
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(project): Form<CreateForm>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .project()
        .create(
            generate_id(),
            project.name,
            organization::id::equals(project.organization_id),
            vec![],
        )
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[instrument(name = "Fetching project", skip_all)]
pub async fn get_project(
    Path(project_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .project()
        .find_first(vec![project::id::equals(project_id)])
        .exec()
        .await
        .unwrap();

    match resp {
        Some(data) => Json(data).into_response(),
        None => (StatusCode::NOT_FOUND, "No such project exists").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateForm {
    name: String,
}

#[instrument(name = "Updating project", skip_all)]
pub async fn update_project(
    Path(project_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(project): Form<UpdateForm>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .project()
        .update(
            project::id::equals(project_id),
            vec![project::name::set(project.name)],
        )
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}
