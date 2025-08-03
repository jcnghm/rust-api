use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

#[derive(Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub fn new(username: String, password: String, role: String) -> Self {
        let password_hash = Self::hash_password(&password);
        Self {
            username,
            password_hash,
            role,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        Self::verify_password_hash(password, &self.password_hash)
    }

    fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string()
    }

    fn verify_password_hash(password: &str, hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(hash).expect("Failed to parse hash");

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
