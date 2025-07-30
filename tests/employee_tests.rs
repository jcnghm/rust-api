mod common;

use actix_web::{App, http::StatusCode, test, web};
use common::{create_test_pool_with_employees, seed_test_employees};
use rust_api_framework::{
    handlers,
    middleware::AuthMiddleware,
    repositories::{EmployeeRepository, ObjectRepository},
    services::{AuthService, EmployeeService, ObjectService},
};
use serde_json::json;

#[actix_web::test]
async fn test_get_employees_success() {
    let pool = create_test_pool_with_employees().await;
    seed_test_employees(&pool).await;

    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(employee_service))
            .app_data(web::Data::new(auth_service.clone()))
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
                    .service(handlers::get_employees)
                    .service(handlers::get_employee)
                    .service(handlers::get_employees_by_store),
            ),
    )
    .await;

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let auth_req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let auth_resp = test::call_service(&app, auth_req).await;
    let auth_body: serde_json::Value = test::read_body_json(auth_resp).await;
    let token = auth_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let req = test::TestRequest::get()
        .uri("/employees")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["employees"].is_array());
    assert_eq!(body["data"]["total"].as_u64().unwrap(), 12);
    assert_eq!(body["data"]["employees"].as_array().unwrap().len(), 10); // default limit
}

#[actix_web::test]
async fn test_get_employees_with_limit() {
    let pool = create_test_pool_with_employees().await;
    seed_test_employees(&pool).await;

    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(employee_service))
            .app_data(web::Data::new(auth_service.clone()))
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
                    .service(handlers::get_employees)
                    .service(handlers::get_employee)
                    .service(handlers::get_employees_by_store),
            ),
    )
    .await;

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let auth_req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let auth_resp = test::call_service(&app, auth_req).await;
    let auth_body: serde_json::Value = test::read_body_json(auth_resp).await;
    let token = auth_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let req = test::TestRequest::get()
        .uri("/employees?limit=5")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["employees"].as_array().unwrap().len(), 5);
    assert_eq!(body["data"]["limit"].as_u64().unwrap(), 5);
}

#[actix_web::test]
async fn test_get_employee_by_id_success() {
    let pool = create_test_pool_with_employees().await;
    seed_test_employees(&pool).await;

    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(employee_service))
            .app_data(web::Data::new(auth_service.clone()))
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
                    .service(handlers::get_employees)
                    .service(handlers::get_employee)
                    .service(handlers::get_employees_by_store),
            ),
    )
    .await;

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let auth_req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let auth_resp = test::call_service(&app, auth_req).await;
    let auth_body: serde_json::Value = test::read_body_json(auth_resp).await;
    let token = auth_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let req = test::TestRequest::get()
        .uri("/employees/1")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_u64().unwrap(), 1);
    assert_eq!(body["data"]["first_name"].as_str().unwrap(), "John");
    assert_eq!(body["data"]["last_name"].as_str().unwrap(), "Smith");
    assert_eq!(body["data"]["store_id"].as_u64().unwrap(), 1);
}

#[actix_web::test]
async fn test_get_employee_by_id_not_found() {
    let pool = create_test_pool_with_employees().await;
    seed_test_employees(&pool).await;

    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(employee_service))
            .app_data(web::Data::new(auth_service.clone()))
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
                    .service(handlers::get_employees)
                    .service(handlers::get_employee)
                    .service(handlers::get_employees_by_store),
            ),
    )
    .await;

    // Get auth token
    let login_data = json!({
        "username": "admin",
        "password": "password123"
    });

    let auth_req = test::TestRequest::post()
        .uri("/token")
        .set_json(&login_data)
        .to_request();

    let auth_resp = test::call_service(&app, auth_req).await;
    let auth_body: serde_json::Value = test::read_body_json(auth_resp).await;
    let token = auth_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let req = test::TestRequest::get()
        .uri("/employees/999")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body["success"].as_bool().unwrap());
    assert_eq!(body["error"].as_str().unwrap(), "Employee not found");
}

#[actix_web::test]
async fn test_employee_endpoints_require_auth() {
    let pool = create_test_pool_with_employees().await;
    seed_test_employees(&pool).await;

    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(object_service))
            .app_data(web::Data::new(employee_service))
            .app_data(web::Data::new(auth_service.clone()))
            // Public routes (no auth middleware)
            .service(handlers::health_check)
            .service(handlers::login)
            // Protected routes (with auth middleware)
            .service(
                web::scope("")
                    .wrap(AuthMiddleware::new(auth_service))
                    .service(handlers::get_employees)
                    .service(handlers::get_employee)
                    .service(handlers::get_employees_by_store),
            ),
    )
    .await;

    // Test without auth token
    let req = test::TestRequest::get().uri("/employees").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Test with invalid token
    let req = test::TestRequest::get()
        .uri("/employees/1")
        .insert_header(("Authorization", "Bearer invalid-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
