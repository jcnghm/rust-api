use actix_web::{get, HttpResponse, Result};
use crate::utils::ApiResponse;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success_no_data("Service is healthy")))
}