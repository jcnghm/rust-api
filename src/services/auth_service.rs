use crate::errors::ApiError;
use crate::models::auth::*;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AuthService {
    users: HashMap<String, User>,
    jwt_secret: String,
    token_duration: Duration,
    refresh_token_duration: Duration,
}

impl AuthService {
    pub fn new() -> Self {
        let mut users = HashMap::new();

        // Add some demo users for testing
        users.insert(
            "admin".to_string(),
            User::new(
                "admin".to_string(),
                "password123".to_string(),
                "admin".to_string(),
            ),
        );
        users.insert(
            "user".to_string(),
            User::new(
                "user".to_string(),
                "userpass".to_string(),
                "user".to_string(),
            ),
        );

        Self {
            users,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            token_duration: Duration::hours(1),
            refresh_token_duration: Duration::days(7),
        }
    }

    pub async fn authenticate(&self, login_req: LoginRequest) -> Result<TokenResponse, ApiError> {
        let user = self
            .users
            .get(&login_req.username)
            .ok_or_else(|| ApiError::BadRequest("Invalid credentials".to_string()))?;

        if !user.verify_password(&login_req.password) {
            return Err(ApiError::BadRequest("Invalid credentials".to_string()));
        }

        let now = Utc::now();

        // Generate claims and access token
        let access_expires_at = now + self.token_duration;
        let access_claims = Claims {
            sub: user.username.clone(),
            exp: access_expires_at.timestamp(),
            iat: now.timestamp(),
            role: user.role.clone(),
        };
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| ApiError::InternalServerError("Failed to create token".to_string()))?;

        // Generate claims and refresh token
        let refresh_expires_at = now + self.refresh_token_duration;
        let refresh_claims = Claims {
            sub: user.username.clone(),
            exp: refresh_expires_at.timestamp(),
            iat: now.timestamp(),
            role: user.role.clone(),
        };
        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| ApiError::InternalServerError("Failed to create refresh token".to_string()))?;

        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.token_duration.num_seconds(),
            refresh_token: Some(refresh_token),
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, ApiError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ApiError::AuthorizationError("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }

    pub async fn refresh_token(&self, refresh_token: String) -> Result<TokenResponse, ApiError> {
        let refresh_claims = self.verify_token(&refresh_token)?;

        // Check if the user still exists
        let user = self
            .users
            .get(&refresh_claims.sub)
            .ok_or_else(|| ApiError::AuthorizationError("User not found".to_string()))?;

        let now = Utc::now();

        // Generate new claims and access token
        let access_expires_at = now + self.token_duration;
        let new_access_claims = Claims {
            sub: user.username.clone(),
            exp: access_expires_at.timestamp(),
            iat: now.timestamp(),
            role: user.role.clone(),
        };
        let new_access_token = encode(
            &Header::default(),
            &new_access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| ApiError::InternalServerError("Failed to create token".to_string()))?;

        // Generate new claims and refresh token
        let refresh_expires_at = now + self.refresh_token_duration;
        let new_refresh_claims = Claims {
            sub: user.username.clone(),
            exp: refresh_expires_at.timestamp(),
            iat: now.timestamp(),
            role: user.role.clone(),
        };
        let new_refresh_token = encode(
            &Header::default(),
            &new_refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| ApiError::InternalServerError("Failed to create refresh token".to_string()))?;

        Ok(TokenResponse {
            access_token: new_access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.token_duration.num_seconds(),
            refresh_token: Some(new_refresh_token),
        })
    }
}
