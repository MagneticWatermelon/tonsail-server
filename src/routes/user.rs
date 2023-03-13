use super::AppState;
use crate::{
    domain::{
        auth::{AuthContext, TonsailUser},
        user::{UserPasswordForm, UserUpdateForm},
    },
    prisma::user,
    util::{
        app_error::AppError,
        hash::{check_hash, hash_password},
        validation::ValidatedForm,
    },
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use tracing::instrument;

#[instrument(name = "Fetching user", skip_all)]
pub async fn get_user(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let data = state
        .db_client
        .user()
        .find_first(vec![user::id::equals(user_id)])
        .exec()
        .await?;

    Ok(Json(data).into_response())
}

#[instrument(name = "Updating user", skip_all)]
pub async fn update_user(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
    ValidatedForm(user): ValidatedForm<UserUpdateForm>,
) -> Result<Response, AppError> {
    let mut params = vec![];
    if user.name.is_some() {
        params.push(user::name::set(user.name.unwrap()));
    }

    if user.email.is_some() {
        params.push(user::email::set(user.email.unwrap()));
    }

    let data = state
        .db_client
        .user()
        .update(user::id::equals(user_id), params)
        .exec()
        .await?;

    Ok(Json(data).into_response())
}

#[instrument(name = "Updating password", skip_all)]
pub async fn update_password(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
    mut auth: AuthContext,
    Extension(user): Extension<TonsailUser>,
    ValidatedForm(password): ValidatedForm<UserPasswordForm>,
) -> Result<Response, AppError> {
    check_hash(password.old.as_bytes(), &user.password)?;
    let hashed = hash_password(password.new.as_bytes());

    let data = state
        .db_client
        .user()
        .update(user::id::equals(user_id), vec![user::password::set(hashed)])
        .exec()
        .await?;

    let user = TonsailUser::from(data.clone());
    match auth.login(&user).await {
        Ok(_) => Ok(Json(data).into_response()),
        Err(_) => Err(AppError::UnAuthorized(
            "Crendentials are invalid".to_string(),
        )),
    }
}
