use actix_web::{get, post, HttpResponse, Result};
use crate::utils::ApiResponse;

// TEST ROUTES
#[get("/")]
pub async fn hello() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success_no_data("Hello world!")))
}

#[post("/echo")]
pub async fn echo(req_body: String) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({"echoed": req_body}),
        "Message echoed"
    )))
}