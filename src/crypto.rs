use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    // Argon2 default parameters
    let argon2 = Argon2::default();

    // Salt is generated for you
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
