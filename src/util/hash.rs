use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

pub fn hash_password(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password, &salt)
        .expect("Could not hash the password")
        .to_string();
    println!("{hash}");
    hash
}

pub fn check_hash(password: &[u8], hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).expect("Could not parse the hash");
    Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok()
}
