use actix_web::{App, HttpServer, middleware::Logger, web};

mod config;
mod database;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod utils;

use config::AppConfig;
use database::create_pool;
use middleware::AuthMiddleware;
use repositories::{EmployeeRepository, ObjectRepository};
use services::{AuthService, EmployeeService, ObjectService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize configuration
    let config = AppConfig::new();

    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&config.log_level));

    // Initialize database
    let pool = create_pool(&config)
        .await
        .expect("Failed to create database pool");

    // Initialize dependencies
    let object_repository = ObjectRepository::new(pool.clone());
    let object_service = ObjectService::new(object_repository);
    let employee_repository = EmployeeRepository::new(pool);
    let employee_service = EmployeeService::new(employee_repository);
    let auth_service = AuthService::new();

    // Print server start message and demo credentials
    println!("API Framework starting...");
    println!("Starting server at http://{}", config.server_address());
    println!("Database: {}", config.database_url);
    println!("Demo credentials:");
    println!("  -- admin::password123 (admin role)");
    println!("  -- user::userpass (user role)");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(object_service.clone()))
            .app_data(web::Data::new(employee_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .wrap(Logger::default())
            .wrap(AuthMiddleware::new(auth_service.clone()))
            // Authentication endpoint (unprotected)
            .service(handlers::login)
            // Protected routes
            // Health check
            .service(handlers::health_check)
            // Test routes (protected)
            .service(handlers::hello)
            .service(handlers::echo)
            .route("/hey", web::get().to(handlers::manual_hello))
            // Object CRUD routes (protected)
            .service(handlers::get_objects)
            .service(handlers::get_object)
            .service(handlers::create_object)
            .service(handlers::update_object)
            .service(handlers::patch_object)
            .service(handlers::delete_object)
            .service(handlers::get_object_profile)
            .service(handlers::get_stats)
            // Employee GET routes (protected)
            .service(handlers::get_employees)
            .service(handlers::get_employee)
            .service(handlers::get_employees_by_store)
    })
    .bind(config.server_address())?
    .run()
    .await
}
