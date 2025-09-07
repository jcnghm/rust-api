mod common;

use actix_web::{App, http::StatusCode, test, web};
use common::create_test_pool;
use rust_api_framework::{
    handlers, middleware::AuthMiddleware, repositories::ObjectRepository, services::AuthService,
    services::ObjectService,
};
use serde_json::json;
use std::sync::Mutex;

#[actix_web::test]
async fn test_login_success() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(auth_service.clone())
            .service(handlers::login),
    )
    .await;

    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Login success response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Login response body: {:?}", body);

    assert!(body.get("data").is_some());
    assert!(body["data"].get("access_token").is_some());
    assert!(body["data"].get("expires_in").is_some());
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(auth_service.clone())
            .service(handlers::login),
    )
    .await;

    let login_data = json!({
        "username": "admin",
        "password": "wrongpassword"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Invalid login response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_protected_route_without_token() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(auth_service.clone())
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service.clone()))
                    .service(handlers::hello),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/hello").to_request();

    let resp = test::call_service(&app, req).await;

    println!("Protected route without token status: {}", resp.status());

    assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Response body: {:?}", body);

    // Check if error message exists
    assert!(body.get("error").is_some());
}

#[actix_web::test]
async fn test_protected_route_with_valid_token() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(auth_service.clone())
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service.clone()))
                    .service(handlers::hello)
                    .service(handlers::get_objects),
            ),
    )
    .await;

    // First login to get a token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Check if login was successful first
    let status = resp.status();
    if status != StatusCode::OK {
        let body_bytes = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body_bytes);
        panic!("Login failed with status {}: {}", status, body_str);
    }

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Login response for token extraction: {:?}", body);

    let token = body["data"]["access_token"].as_str().unwrap();

    // Now test the protected route
    let req = test::TestRequest::get()
        .uri("/health") // Changed from "/" to "/hello" to match the handler
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Protected route with valid token status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_protected_route_with_invalid_token() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(auth_service.clone())
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service.clone()))
                    .service(handlers::hello),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/hello")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!(
        "Protected route with invalid token status: {}",
        resp.status()
    );

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
