use actix_web::{App, HttpServer, middleware::Logger, web};
use num_cpus;
use std::sync::Mutex;
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
use repositories::{EmployeeRepository, ObjectRepository, TaskRepository};
use services::{AuthService, EmployeeService, ObjectService, TaskService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::new();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&config.log_level));

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
    let employee_service = Arc::new(EmployeeService::new(EmployeeRepository::new(pool.clone())));
    let task_service = Arc::new(TaskService::new(TaskRepository::new(pool.clone())));
    let auth_service = web::Data::new(Mutex::new(AuthService::new()));
    let workers = num_cpus::get();

    println!(
        "--------------------------------\
            \nStarting API...\
            \nUsing {} workers\
            \n--------------------------------\
            \nServer running at http://{}\
            \nDatabase: {}\
            \n--------------------------------",
        workers,
        config.server_address(),
        config.database_url
    );

    // TEST COMMENT
    let server_addr = config.server_address();

    let object_service_data = web::Data::from(object_service);
    let employee_service_data = web::Data::from(employee_service);
    let task_service_data = web::Data::from(task_service.clone());
    let auth_service_data = web::Data::new(Mutex::new(AuthService::new()));

    // Create and run the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(object_service_data.clone())
            .app_data(employee_service_data.clone())
            .app_data(task_service_data.clone())
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
        .service(handlers::refresh_token)
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
                .service(handlers::get_employees_by_store)
                .service(handlers::create_employees),
        )
        .service(
            web::scope("/tasks")
                .service(handlers::get_tasks)
                .service(handlers::get_task)
                .service(handlers::create_task)
                .service(handlers::update_task)
                .service(handlers::delete_task)
                .service(handlers::assign_task),
        );
}
