use actix_web::{get, HttpResponse, Result};
use crate::utils::ApiResponse;
use chrono::Utc;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "current_time_utc": Utc::now().to_string() }),
        "Service is healthy"
    )))
}