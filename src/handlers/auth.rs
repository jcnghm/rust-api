use actix_web::{post, web, HttpResponse, Result, ResponseError};
use crate::services::AuthService;
use crate::models::auth::LoginRequest;
use crate::utils::ApiResponse;

#[post("/token")]
pub async fn login(
    auth_service: web::Data<AuthService>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    match auth_service.authenticate(login_req.into_inner()).await {
        Ok(token_response) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(token_response, "Login successful")))
        }
        Err(e) => Ok(e.error_response()),
    }
}