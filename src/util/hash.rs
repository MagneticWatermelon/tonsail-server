use super::app_error::AppError;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

pub fn hash_password(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .expect("Could not hash the password")
        .to_string()
}

pub fn check_hash(password: &[u8], hash: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(hash).expect("Could not parse the hash");
    match Argon2::default().verify_password(password, &parsed_hash) {
        Ok(_) => Ok(()),
        Err(_) => Err(AppError::UnAuthorized(format!("Wrong credentials"))),
    }
}
