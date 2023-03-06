use super::auth::validate_password;
use super::{MAX_NAME_LENGTH, MIN_NAME_LENGTH};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct UserUpdateForm {
    #[validate(length(min = "MIN_NAME_LENGTH", max = "MAX_NAME_LENGTH"))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UserPasswordForm {
    pub old: String,
    #[validate(custom(function = "validate_password"))]
    pub new: String,
}
