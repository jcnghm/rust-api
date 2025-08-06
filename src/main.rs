use actix_web::{App, HttpServer, middleware::Logger, web};
use num_cpus;
use std::{sync::Arc, time::Duration};

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

    // Print configuration and start server message
    println!("--------------------------------");
    println!(
        "Logging initialized with level: {}",
        config.log_level.to_uppercase()
    );

    println!("--------------------------------");
    println!("Initializing database...");
    let pool = create_pool(&config)
        .await
        .expect("Failed to create database pool");

    println!("Initializing services and repositories...");
    let object_service = Arc::new(ObjectService::new(ObjectRepository::new(pool.clone())));
    let employee_service = Arc::new(EmployeeService::new(EmployeeRepository::new(pool)));
    let auth_service = AuthService::new();

    println!(
        "--------------------------------\
             \nStarting API...\
             \nServer running at http://{}\
             \nDatabase: {}\
             \nDemo credentials:\
             \n  -- admin::password123 (admin role)\
             \n  -- user::userpass (user role)\
             \n--------------------------------",
        config.server_address(),
        config.database_url
    );

    let server_addr = config.server_address();

    let object_service_data = web::Data::from(object_service);
    let employee_service_data = web::Data::from(employee_service);
    let auth_service_data = web::Data::new(auth_service.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(object_service_data.clone())
            .app_data(employee_service_data.clone())
            .app_data(auth_service_data.clone())
            .wrap(Logger::default())
            .wrap(AuthMiddleware::new(auth_service.clone()))
            .configure(configure_routes)
    })
    .workers(num_cpus::get())
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(30))
    .bind(server_addr)?
    .run()
    .await
}

fn configure_routes(config: &mut web::ServiceConfig) {
    config
        .service(handlers::login)
        .service(handlers::health_check)
        .service(
            web::scope("/objects")
                .service(handlers::get_objects)
                .service(handlers::get_object)
                .service(handlers::create_object)
                .service(handlers::update_object)
                .service(handlers::patch_object)
                .service(handlers::delete_object),
        )
        .service(
            web::scope("/employees")
                .service(handlers::get_employees)
                .service(handlers::get_employee)
                .service(handlers::get_employees_by_store),
        );
}
