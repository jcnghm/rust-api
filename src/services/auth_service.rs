use crate::models::auth::*;
use crate::errors::ApiError;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AuthService {
    users: HashMap<String, User>,
    jwt_secret: String,
    token_duration: Duration,
}

impl AuthService {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        
        // Add some demo users
        users.insert(
            "admin".to_string(),
            User::new("admin".to_string(), "password123".to_string(), "admin".to_string())
        );
        users.insert(
            "user".to_string(),
            User::new("user".to_string(), "userpass".to_string(), "user".to_string())
        );

        Self {
            users,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            token_duration: Duration::hours(1),
        }
    }

    pub async fn authenticate(&self, login_req: LoginRequest) -> Result<TokenResponse, ApiError> {
        let user = self.users.get(&login_req.username)
            .ok_or_else(|| ApiError::BadRequest("Invalid credentials".to_string()))?;

        if !user.verify_password(&login_req.password) {
            return Err(ApiError::BadRequest("Invalid credentials".to_string()));
        }

        let now = Utc::now();
        let expires_at = now + self.token_duration;
        
        let claims = Claims {
            sub: user.username.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            role: user.role.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref())
        ).map_err(|_| ApiError::InternalServerError("Failed to create token".to_string()))?;

        Ok(TokenResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: self.token_duration.num_seconds(),
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, ApiError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256)
        ).map_err(|_| ApiError::AuthorizationError("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }
}