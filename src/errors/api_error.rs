use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
    ValidationError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    code: u16,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: msg.clone(),
                code: 400,
            }),
            ApiError::NotFound(msg) => HttpResponse::NotFound().json(ErrorResponse {
                success: false,
                error: msg.clone(),
                code: 404,
            }),
            ApiError::InternalServerError(msg) => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    success: false,
                    error: msg.clone(),
                    code: 500,
                })
            }
            ApiError::ValidationError(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: msg.clone(),
                code: 400,
            }),
        }
    }
}
