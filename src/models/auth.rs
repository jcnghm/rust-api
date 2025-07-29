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
    pub expires_in: i64, // seconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // subject (username)
    pub exp: i64,         // expiration time
    pub iat: i64,         // issued at
    pub role: String,     // user role
}

#[derive(Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String, // In production, this would be properly hashed
    pub role: String,
}

impl User {
    pub fn new(username: String, password: String, role: String) -> Self {
        // In production, use proper password hashing like bcrypt
        let password_hash = format!("hash_{}", password); // Simplified for demo
        Self {
            username,
            password_hash,
            role,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        // In production, use proper password verification
        self.password_hash == format!("hash_{}", password)
    }
}