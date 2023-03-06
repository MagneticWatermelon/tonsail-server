use super::AppState;
use crate::domain::auth::{AuthContext, AuthLoginForm, AuthRegisterForm, TonsailUser};
use crate::prisma::{organization, user};
use crate::util::hash::{check_hash, hash_password};
use crate::util::http_error::HttpError;
use crate::util::nano_id::generate_id;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use http::StatusCode;
use prisma_client_rust::QueryError;
use tracing::instrument;

pub async fn check_me(auth: AuthContext) -> Response {
    if let Some(user) = auth.current_user {
        return Json(user).into_response();
    }
    HttpError::new(
        StatusCode::UNAUTHORIZED,
        "Not Authorized",
        "Token not found",
    )
    .into_response()
}

#[instrument(name = "User attempting to logout", skip_all)]
pub async fn logout(mut auth: AuthContext) -> StatusCode {
    auth.logout().await;
    StatusCode::OK
}

#[instrument(name = "User attempting to login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    mut auth: AuthContext,
    Form(user): Form<AuthLoginForm>,
) -> Response {
    let resp = state
        .db_client
        .user()
        .find_first(vec![user::email::equals(user.email)])
        .exec()
        .await
        .unwrap();

    if resp.is_some() {
        let data = resp.unwrap();
        if !check_hash(user.password.as_bytes(), &data.password) {
            HttpError::new(
                StatusCode::UNAUTHORIZED,
                "Login failed",
                "Unable to login with provided credentials",
            )
            .into_response()
        } else {
            let user = TonsailUser::from(data.clone());
            match auth.login(&user).await {
                Ok(_) => (StatusCode::OK, Json(user)).into_response(),
                Err(_) => (StatusCode::UNAUTHORIZED, "").into_response(),
            }
        }
    } else {
        HttpError::new(StatusCode::UNAUTHORIZED, "Login failed", "Unable to login").into_response()
    }
}

#[instrument(name = "Registering new user", skip_all)]
pub async fn register_new_user(
    State(state): State<AppState>,
    Form(user): Form<AuthRegisterForm>,
) -> Response {
    match create_user(State(state), Form(user)).await {
        Ok((_new_org, new_user)) => Json(new_user).into_response(),
        Err(e) => HttpError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Register failed",
            e.to_string().as_str(),
        )
        .into_response(),
    }
}

#[instrument(name = "Writing new user with new organization to database", skip_all)]
async fn create_user(
    State(state): State<AppState>,
    Form(user): Form<AuthRegisterForm>,
) -> Result<(organization::Data, user::Data), QueryError> {
    let (org, user) = state
        .db_client
        ._transaction()
        .run(|client| async move {
            let new_org = client
                .organization()
                .create(generate_id(), user.email.clone(), vec![])
                .exec()
                .await?;

            client
                .user()
                .create(
                    generate_id(),
                    user.email,
                    hash_password(user.password.as_bytes()),
                    user.name,
                    organization::id::equals(new_org.id.clone()),
                    vec![],
                )
                .exec()
                .await
                .map(|user| (new_org, user))
        })
        .await?;

    Ok((org, user))
}
