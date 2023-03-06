use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;
use validator::{Validate, ValidationError};

const MIN_NAME_LENGTH: u8 = 2;
const MAX_NAME_LENGTH: u8 = 90;

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

fn validate_password(password: &str) -> Result<(), ValidationError> {
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
