use crate::utils::ApiResponse;
use actix_web::{HttpResponse, Result, get};
use chrono::Utc;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "current_time_utc": Utc::now().to_string() }),
        "Service is healthy",
    )))
}
