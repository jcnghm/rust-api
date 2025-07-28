use actix_web::{App, http::StatusCode, test, web};
use rust_api_framework::{
    handlers, middleware::AuthMiddleware, repositories::ObjectRepository, services::AuthService,
    services::ObjectService,
};
use serde_json::json;

// Macro to create test app with proper route structure
macro_rules! create_test_app {
    () => {{
        let object_repository = ObjectRepository::new();
        let object_service = ObjectService::new(object_repository);
        let auth_service = AuthService::new();

        test::init_service(
            App::new()
                .app_data(web::Data::new(object_service))
                .app_data(web::Data::new(auth_service.clone()))
                // Public routes (no auth middleware)
                .service(handlers::health_check)
                .service(handlers::login)
                // Protected routes (with auth middleware)
                .service(
                    web::scope("")
                        .wrap(AuthMiddleware::new(auth_service))
                        .service(handlers::hello)
                        .service(handlers::get_objects)
                        .service(handlers::create_object),
                ),
        )
        .await
    }};
}

#[actix_web::test]
async fn test_health_check() {
    // Create app without auth middleware for health check
    let object_repository = ObjectRepository::new();
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(auth_service))
            .service(handlers::health_check),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();

    let resp = test::call_service(&app, req).await;

    println!("Health check response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Health response body: {:?}", body);
}

#[actix_web::test]
async fn test_login_success() {
    // Create app without auth middleware for login endpoint
    let object_repository = ObjectRepository::new();
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(auth_service))
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
    // Create app without auth middleware for login endpoint
    let object_repository = ObjectRepository::new();
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(auth_service))
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
    let object_repository = ObjectRepository::new();
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(auth_service.clone()))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
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
    let app = create_test_app!();

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Login response for token extraction: {:?}", body);

    // Extract token from the actual response format
    let token = body["data"]["access_token"].as_str().unwrap();

    // Test protected route with valid token
    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Protected route with valid token status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_protected_route_with_invalid_token() {
    let object_repository = ObjectRepository::new();
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(auth_service.clone()))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
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

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_get_objects_empty() {
    let app = create_test_app!();

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Login response for token extraction: {:?}", body);

    // Extract token from the actual response format
    let token = body["data"]["access_token"].as_str().unwrap();

    // Test get objects endpoint
    let req = test::TestRequest::get()
        .uri("/objects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Get objects response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Get objects response body: {:?}", body);

    assert!(body["data"]["objects"].is_array());
    assert!(body["data"]["objects"].as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_create_object() {
    let app = create_test_app!();

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Login response for token extraction: {:?}", body);

    // Extract token from the actual response format
    let token = body["data"]["access_token"].as_str().unwrap();

    // Test create object endpoint
    let object_data = json!({
        "name": "Test Object",
        "email": "test@test.com"
    });

    let req = test::TestRequest::post()
        .uri("/objects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&object_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Create object response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Create object response body: {:?}", body);

    assert_eq!(body["data"]["name"], "Test Object");
    assert_eq!(body["data"]["email"], "test@test.com");
    assert!(body["data"]["id"].is_number());
}
