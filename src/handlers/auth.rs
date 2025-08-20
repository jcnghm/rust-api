use crate::models::auth::LoginRequest;
use crate::services::AuthService;
use crate::utils::ApiResponse;
use actix_web::{HttpResponse, ResponseError, Result, post, web};

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
