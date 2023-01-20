use super::AppState;
use crate::prisma::{organization, user, PrismaClient};
use crate::util::hash::{check_hash, hash_password};
use crate::util::nano_id::generate_id;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Form, Json};
use axum_login::secrecy::SecretVec;
use axum_login::{AuthUser, UserStore};
use http::StatusCode;
use prisma_client_rust::QueryError;
use serde::Deserialize;
use std::sync::Arc;
use tracing::instrument;

type AuthContext = axum_login::extractors::AuthContext<TonsailUser, TonsailUserStore>;

#[derive(Debug, Clone, Deserialize)]
pub struct TonsailUser {
    id: String,
    password: String,
}

#[derive(Debug, Clone)]
pub struct TonsailUserStore {
    db_client: Arc<PrismaClient>,
}

impl TonsailUserStore {
    pub fn new(client: Arc<PrismaClient>) -> Self {
        Self { db_client: client }
    }
}

#[async_trait]
impl<Role> UserStore<Role> for TonsailUserStore
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    type User = TonsailUser;

    async fn load_user(&self, user_id: &str) -> eyre::Result<Option<Self::User>> {
        let user = self
            .db_client
            .user()
            .find_unique(user::id::equals(String::from(user_id)))
            .exec()
            .await?;

        match user {
            Some(u) => Ok(Some(TonsailUser {
                id: u.id,
                password: u.password,
            })),
            None => Ok(None),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, PartialOrd)]
// pub enum Role {
//     User,
//     Admin,
// }

impl<Role> AuthUser<Role> for TonsailUser
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    fn get_id(&self) -> String {
        self.id.to_string()
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password.clone().into())
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[instrument(name = "User attempting to login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    mut auth: AuthContext,
    Form(user): Form<LoginForm>,
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
            (StatusCode::UNAUTHORIZED, "Unable to login").into_response()
        } else {
            let user = TonsailUser {
                id: data.id.clone(),
                password: data.password.clone(),
            };
            match auth.login(&user).await {
                Ok(_) => (StatusCode::OK, Json(data)).into_response(),
                Err(_) => (StatusCode::UNAUTHORIZED, "").into_response(),
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, "Unable to login").into_response()
    }
}

#[derive(Deserialize)]
pub struct RegisterForm {
    name: String,
    email: String,
    password: String,
}

#[instrument(name = "Registering new user", skip_all)]
pub async fn register_new_user(
    State(state): State<AppState>,
    Form(user): Form<RegisterForm>,
) -> Response {
    match create_user(State(state), Form(user)).await {
        Ok((_new_org, new_user)) => Json(new_user).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[instrument(name = "Writing new user with new organization to database", skip_all)]
async fn create_user(
    State(state): State<AppState>,
    Form(user): Form<RegisterForm>,
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
