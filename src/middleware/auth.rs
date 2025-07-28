use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, ResponseError,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use crate::services::AuthService;
use crate::errors::ApiError;

pub struct AuthMiddleware {
    auth_service: AuthService,
}

impl AuthMiddleware {
    pub fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;  // Changed this line
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
    auth_service: AuthService,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;  // Changed this line
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_service = self.auth_service.clone();
        
        // Skip authentication for login endpoint and health check
        let path = req.path();
        if path == "/token" {
            let fut = self.service.call(req);
            return Box::pin(async move { 
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        // Extract Authorization header
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(header) => {
                match header.to_str() {
                    Ok(header_str) => {
                        if header_str.starts_with("Bearer ") {
                            &header_str[7..] // Remove "Bearer " prefix
                        } else {
                            return Box::pin(async move {
                                let response = ApiError::BadRequest("Invalid authorization header format".to_string())
                                    .error_response()
                                    .map_into_right_body();  // Map to EitherBody::Right
                                Ok(ServiceResponse::new(req.into_parts().0, response))
                            });
                        }
                    }
                    Err(_) => {
                        return Box::pin(async move {
                            let response = ApiError::BadRequest("Invalid authorization header".to_string())
                                .error_response()
                                .map_into_right_body();  // Map to EitherBody::Right
                            Ok(ServiceResponse::new(req.into_parts().0, response))
                        });
                    }
                }
            }
            None => {
                return Box::pin(async move {
                    let response = ApiError::BadRequest("Missing authorization header".to_string())
                        .error_response()
                        .map_into_right_body();  // Map to EitherBody::Right
                    Ok(ServiceResponse::new(req.into_parts().0, response))
                });
            }
        };

        // Verify token
        match auth_service.verify_token(token) {
            Ok(claims) => {
                // Add claims to request extensions for use in handlers
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move { 
                    let res = fut.await?;
                    Ok(res.map_into_left_body())  // Map to EitherBody::Left
                })
            }
            Err(e) => {
                Box::pin(async move {
                    let response = e.error_response()
                        .map_into_right_body();  // Map to EitherBody::Right
                    Ok(ServiceResponse::new(req.into_parts().0, response))
                })
            }
        }
    }
}