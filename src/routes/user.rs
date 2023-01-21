use super::AppState;
use crate::{prisma::user, util::hash::hash_password};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form, Json,
};
use http::{HeaderMap, StatusCode};
use serde::Deserialize;
use tracing::instrument;

#[instrument(name = "Fetching user", skip_all)]
pub async fn get_user(
    Path(user_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let resp = state
        .db_client
        .user()
        .find_first(vec![user::id::equals(user_id)])
        .exec()
        .await
        .unwrap();

    match resp {
        Some(data) => Json(data).into_response(),
        None => (StatusCode::NOT_FOUND, "No such user exists").into_response(),
    }
}

#[derive(Deserialize)]
pub struct UpdateForm {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[instrument(name = "Updating user", skip_all)]
pub async fn update_user(
    Path(user_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
    Form(user): Form<UpdateForm>,
) -> Response {
    // This can not fail as this is set in middleware
    let _request_id = headers.get("x-request-id").unwrap();

    let mut params = vec![];
    if user.name.is_some() {
        params.push(user::name::set(user.name.unwrap()));
    }
    if user.password.is_some() {
        let hashed = hash_password(user.password.unwrap().as_bytes());
        params.push(user::password::set(hashed));
    }
    if user.email.is_some() {
        params.push(user::email::set(user.email.unwrap()));
    }

    let resp = state
        .db_client
        .user()
        .update(user::id::equals(user_id), params)
        .exec()
        .await;

    match resp {
        Ok(data) => Json(data).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}
