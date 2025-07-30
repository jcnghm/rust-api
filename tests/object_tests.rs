mod common;

use actix_web::{App, http::StatusCode, test, web};
use common::create_test_pool;
use rust_api_framework::{
    handlers, middleware::AuthMiddleware, repositories::ObjectRepository, services::AuthService,
    services::ObjectService,
};
use serde_json::json;

#[actix_web::test]
async fn test_health_check() {
    // Create app without auth middleware for health check
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
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
async fn test_get_objects_empty() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
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
    .await;

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
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
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
    .await;

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

#[actix_web::test]
async fn test_create_and_get_objects() {
    let pool = create_test_pool().await;
    let object_repository = ObjectRepository::new(pool);
    let object_service = ObjectService::new(object_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
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
    .await;

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
    let token = body["data"]["access_token"].as_str().unwrap();

    // Create an object
    let object_data = json!({
        "name": "Integration Test Object",
        "email": "integration@test.com"
    });

    let req = test::TestRequest::post()
        .uri("/objects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&object_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Now get objects and verify the created object is there
    let req = test::TestRequest::get()
        .uri("/objects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Get objects after creation response body: {:?}", body);

    assert!(body["data"]["objects"].is_array());
    let objects = body["data"]["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0]["name"], "Integration Test Object");
    assert_eq!(objects[0]["email"], "integration@test.com");
}
