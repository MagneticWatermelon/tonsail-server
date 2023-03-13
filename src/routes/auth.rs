use super::AppState;
use crate::domain::auth::{AuthContext, AuthLoginForm, AuthRegisterForm, TonsailUser};
use crate::prisma::{organization, user};
use crate::util::app_error::AppError;
use crate::util::hash::{check_hash, hash_password};
use crate::util::nano_id::generate_id;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use http::StatusCode;
use prisma_client_rust::QueryError;
use tracing::instrument;

pub async fn check_me(auth: AuthContext) -> Result<Response, AppError> {
    match auth.current_user {
        Some(user) => Ok(Json(user).into_response()),
        None => Err(AppError::UnAuthorized("Token not found".to_string())),
    }
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
) -> Result<Response, AppError> {
    let resp = state
        .db_client
        .user()
        .find_first(vec![user::email::equals(user.email)])
        .exec()
        .await?;

    match resp {
        Some(data) => {
            check_hash(user.password.as_bytes(), &data.password)?;
            let user = TonsailUser::from(data);
            match auth.login(&user).await {
                Ok(_) => Ok((StatusCode::OK, Json(user)).into_response()),
                Err(_) => Err(AppError::UnAuthorized("Unable to login".to_string())),
            }
        }
        _ => Err(AppError::UnAuthorized("Unable to login".to_string())),
    }
}

#[instrument(name = "Registering new user", skip_all)]
pub async fn register_new_user(
    State(state): State<AppState>,
    Form(user): Form<AuthRegisterForm>,
) -> Result<Response, AppError> {
    let user = create_user(State(state), Form(user)).await?;
    Ok(Json(user).into_response())
}

#[instrument(name = "Writing new user with new organization to database", skip_all)]
async fn create_user(
    State(state): State<AppState>,
    Form(user): Form<AuthRegisterForm>,
) -> Result<user::Data, QueryError> {
    let (_org, user) = state
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

    Ok(user)
}
