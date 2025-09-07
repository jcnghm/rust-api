use crate::models::auth::{LoginRequest, RefreshTokenRequest};
use crate::services::AuthService;
use crate::utils::ApiResponse;
use actix_web::{HttpResponse, ResponseError, Result, post, web};
use std::sync::Mutex;

#[post("/token")]
pub async fn login(
    auth_service: web::Data<Mutex<AuthService>>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let service = auth_service.lock().unwrap();
    match service.authenticate(login_req.into_inner()).await {
        Ok(token_response) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(token_response, "Login successful")))
        }
        Err(e) => Ok(e.error_response()),
    }
}

#[post("/refresh")]
pub async fn refresh_token(
    auth_service: web::Data<Mutex<AuthService>>,
    refresh_req: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse> {
    let service = auth_service.lock().unwrap();
    match service
        .refresh_token(refresh_req.refresh_token.clone())
        .await
    {
        Ok(token_response) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            token_response,
            "Token refreshed successfully",
        ))),
        Err(e) => Ok(e.error_response()),
    }
}
