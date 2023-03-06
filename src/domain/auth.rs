use super::{MAX_NAME_LENGTH, MIN_NAME_LENGTH};
use crate::prisma::{organization, user, PrismaClient};
use axum::async_trait;
use axum_login::{secrecy::SecretVec, AuthUser, UserStore};
use prisma_client_rust::chrono;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use validator::{Validate, ValidationError};

pub type AuthContext = axum_login::extractors::AuthContext<TonsailUser, TonsailUserStore>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonsailUser {
    id: String,
    name: String,
    email: String,
    pub password: String,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::FixedOffset>,
    #[serde(rename = "updatedAt")]
    updated_at: chrono::DateTime<chrono::FixedOffset>,
    organization: Option<Box<organization::Data>>,
    #[serde(rename = "organizationId")]
    organization_id: String,
}

impl From<user::Data> for TonsailUser {
    fn from(u: user::Data) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            password: u.password,
            created_at: u.created_at,
            updated_at: u.updated_at,
            organization: u.organization,
            organization_id: u.organization_id,
        }
    }
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
            Some(u) => Ok(Some(TonsailUser::from(u))),
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

#[derive(Debug, Validate, Deserialize)]
pub struct AuthLoginForm {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct AuthRegisterForm {
    #[validate(length(min = "MIN_NAME_LENGTH", max = "MAX_NAME_LENGTH"))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let is_empty = password.trim().is_empty();
    let is_too_short = password.graphemes(true).count() < 8;
    let does_not_contains_number = !password.chars().any(|g| g.is_digit(10));
    let does_not_contains_lowercase = !password.chars().any(|g| g.is_lowercase());
    let does_not_contains_uppercase = !password.chars().any(|g| g.is_uppercase());
    let does_not_contains_special = !password.chars().any(|g| g.is_ascii_punctuation());

    if is_empty
        || is_too_short
        || does_not_contains_number
        || does_not_contains_special
        || does_not_contains_uppercase
        || does_not_contains_lowercase
    {
        return Err(ValidationError::new(
            "Password does not meet strength requirements",
        ));
    }
    Ok(())
}
