use super::AppState;
use crate::{
    domain::organization::UpdateForm,
    prisma::organization,
    util::{app_error::AppError, validation::ValidatedForm},
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use tracing::instrument;

#[instrument(name = "Fetching all organizations", skip_all)]
pub async fn get_organizations(State(state): State<AppState>) -> Result<Response, AppError> {
    let data = state
        .db_client
        .organization()
        .find_many(vec![])
        .exec()
        .await?;

    Ok(Json(data).into_response())
}

#[instrument(name = "Fetching organization", skip_all)]
pub async fn get_organization(
    Path(org_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let resp = state
        .db_client
        .organization()
        .find_first(vec![organization::id::equals(org_id)])
        .with(organization::users::fetch(vec![]))
        .with(organization::projects::fetch(vec![]))
        .exec()
        .await?;

    match resp {
        Some(data) => Ok(Json(data).into_response()),
        None => Ok((StatusCode::NO_CONTENT, Json(())).into_response()),
    }
}

#[instrument(name = "Updating organization", skip_all)]
pub async fn update_organization(
    Path(org_id): Path<String>,
    State(state): State<AppState>,
    ValidatedForm(org): ValidatedForm<UpdateForm>,
) -> Result<Response, AppError> {
    let data = state
        .db_client
        .organization()
        .update(
            organization::id::equals(org_id),
            vec![organization::name::set(org.name)],
        )
        .exec()
        .await?;

    Ok(Json(data).into_response())
}
