use crate::errors::ApiError;
use crate::services::AuthService;
use actix_web::{
    Error, HttpMessage, ResponseError, web,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::sync::Mutex;

pub struct AuthMiddleware {
    auth_service: web::Data<Mutex<AuthService>>,
}

impl AuthMiddleware {
    pub fn new(auth_service: web::Data<Mutex<AuthService>>) -> Self {
        Self { auth_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            auth_service: self.auth_service.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    auth_service: web::Data<Mutex<AuthService>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_service = self.auth_service.clone();

        let path = req.path();
        if path == "/token" || path == "/refresh" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(header) => match header.to_str() {
                Ok(header_str) => {
                    if header_str.starts_with("Bearer ") {
                        &header_str[7..]
                    } else {
                        return Box::pin(async move {
                            let response = ApiError::BadRequest(
                                "Invalid authorization header format".to_string(),
                            )
                            .error_response()
                            .map_into_right_body();
                            Ok(ServiceResponse::new(req.into_parts().0, response))
                        });
                    }
                }
                Err(_) => {
                    return Box::pin(async move {
                        let response =
                            ApiError::BadRequest("Invalid authorization header".to_string())
                                .error_response()
                                .map_into_right_body();
                        Ok(ServiceResponse::new(req.into_parts().0, response))
                    });
                }
            },
            None => {
                return Box::pin(async move {
                    let response = ApiError::BadRequest("Missing authorization header".to_string())
                        .error_response()
                        .map_into_right_body();
                    Ok(ServiceResponse::new(req.into_parts().0, response))
                });
            }
        };

        let token_str = token.to_string();
        // Verify token before entering async block
        let verification_result = {
            let service_guard = auth_service.lock().unwrap();
            service_guard.verify_token(&token_str)
        };

        match verification_result {
            Ok(claims) => {
                // Add claims to request extensions for use in handlers
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_left_body())
                })
            }
            Err(e) => Box::pin(async move {
                let response = e.error_response().map_into_right_body();
                Ok(ServiceResponse::new(req.into_parts().0, response))
            }),
        }
    }
}