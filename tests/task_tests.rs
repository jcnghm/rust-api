mod common;

use actix_web::{App, http::StatusCode, test, web};
use common::create_test_pool;
use rust_api_framework::{
    handlers, middleware::AuthMiddleware, repositories::TaskRepository, services::AuthService,
    services::TaskService,
};
use serde_json::json;

#[actix_web::test]
async fn test_get_tasks_empty() {
    let pool = create_test_pool().await;
    let task_repository = TaskRepository::new(pool);
    let task_service = TaskService::new(task_repository);
    let auth_service: AuthService = AuthService::new();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(task_service))
            .app_data(web::Data::new(auth_service.clone()))
            .wrap(AuthMiddleware::new(auth_service.clone()))
            .service(handlers::login)
            .service(web::scope("/tasks").service(handlers::get_tasks)),
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

    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["data"]["access_token"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri("/tasks/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    println!("Get objects response status: {}", resp.status());

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Get objects response body: {:?}", body);

    assert!(body["data"]["tasks"].is_array());
    assert!(body["data"]["tasks"].as_array().unwrap().is_empty());
}
