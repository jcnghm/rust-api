use actix_web::{
    App, HttpResponse, HttpServer, Responder, Result, delete, get,
    middleware::Logger,
    patch, post, put,
    web::{self, Json, Path, Query},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Data models
#[derive(Serialize, Deserialize, Clone)]
struct Object {
    id: u32,
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Deserialize)]
struct CreateObjectRequest {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Deserialize)]
struct UpdateObjectRequest {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

#[derive(Deserialize)]
struct ObjectQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    name: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    code: u16,
}

// Helper functions
impl<T> ApiResponse<T> {
    fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
        }
    }
}

impl ApiResponse<()> {
    fn success_no_data(message: &str) -> Self {
        Self {
            success: true,
            data: Some(()),
            message: message.to_string(),
        }
    }
}

impl ErrorResponse {
    fn new(error: &str, code: u16) -> Self {
        Self {
            success: false,
            error: error.to_string(),
            code,
        }
    }
}

/**
 * In-memory storage for development - REPLACE WITH DATABASE IN PRODUCTION
 *
 * To set up MySQL database integration with migrations:
 *
 * 1. Install SQLx CLI and create database:
 *    ```bash
 *    cargo install sqlx-cli --no-default-features --features mysql
 *    mysql -u root -p -e "CREATE DATABASE your_app_db;"
 *    mysql -u root -p -e "CREATE USER 'app_user'@'localhost' IDENTIFIED BY 'secure_password';"
 *    mysql -u root -p -e "GRANT ALL PRIVILEGES ON your_app_db.* TO 'app_user'@'localhost';"
 *    ```
 *
 * 2. Add dependencies to Cargo.toml:
 *    ```toml
 *    [dependencies]
 *    sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "mysql", "chrono", "uuid", "migrate"] }
 *    chrono = { version = "0.4", features = ["serde"] }
 *    ```
 *
 * 3. Set environment variable and initialize migrations:
 *    ```bash
 *    export DATABASE_URL="mysql://app_user:secure_password@localhost/your_app_db"
 *    sqlx migrate add create_objects_table
 *    ```
 *
 * 4. Edit the generated migration file (migrations/XXXXXX_create_objects_table.sql):
 *    ```sql
 *    -- migrations/XXXXXX_create_objects_table.sql
 *    CREATE TABLE objects (
 *        id INT AUTO_INCREMENT PRIMARY KEY,
 *        name VARCHAR(255) NOT NULL,
 *        email VARCHAR(255) NOT NULL UNIQUE,
 *        age INT NULL,
 *        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
 *        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
 *        INDEX idx_objects_name (name),
 *        INDEX idx_objects_email (email)
 *    );
 *    ```
 *
 * 5. Run migrations:
 *    ```bash
 *    sqlx migrate run
 *    ```
 *
 * 6. Replace ObjectStore type with:
 *    ```rust
 *    type DbPool = web::Data<sqlx::MySqlPool>;
 *    ```
 *
 * 7. Initialize connection pool and run migrations in main():
 *    ```rust
 *    let database_url = env::var("DATABASE_URL")
 *        .expect("DATABASE_URL must be set");
 *    let pool = sqlx::MySqlPool::connect(&database_url).await?;
 *    
 *    // Run migrations on startup
 *    sqlx::migrate!("./migrations").run(&pool).await?;
 *    ```
 *
 * 8. Update Object struct to work with database:
 *    ```rust
 *    #[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
 *    struct Object {
 *        id: u32,
 *        name: String,
 *        email: String,
 *        age: Option<u32>,
 *        created_at: chrono::DateTime<chrono::Utc>,
 *        updated_at: chrono::DateTime<chrono::Utc>,
 *    }
 *    ```
 *
 * 9. Example handler function with SQL:
 *    ```rust
 *    async fn get_objects(pool: DbPool, query: Query<ObjectQuery>) -> Result<impl Responder> {
 *        let limit = query.limit.unwrap_or(10) as i64;
 *        let offset = query.offset.unwrap_or(0) as i64;
 *        
 *        let objects = sqlx::query_as!(
 *            Object,
 *            "SELECT id, name, email, age, created_at, updated_at FROM objects LIMIT ? OFFSET ?",
 *            limit, offset
 *        ).fetch_all(pool.get_ref()).await.map_err(|_| HttpResponse::InternalServerError())?;
 *        
 *        // ... rest of handler logic
 *    }
 *    ```
 *
 * 10. Additional useful migration commands:
 *     ```bash
 *     sqlx migrate info                    # Show migration status
 *     sqlx migrate revert                  # Revert last migration
 *     sqlx migrate add add_index_to_objects # Add new migration
 *     ```
 *
 * For complete SQLx migration guide: https://docs.rs/sqlx/latest/sqlx/migrate/index.html
 */

// In-memory storage (replace with database in production)
type ObjectStore = web::Data<std::sync::Mutex<HashMap<u32, Object>>>;

// Basic endpoints
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success_no_data("Hello world!"))
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success_no_data("Service is healthy"))
}

// Object CRUD endpoints
#[get("/objects")]
async fn get_objects(store: ObjectStore, query: Query<ObjectQuery>) -> Result<impl Responder> {
    let objects = store.lock().unwrap();
    let mut object_list: Vec<Object> = objects.values().cloned().collect();

    // Filter by name if provided
    if let Some(name) = &query.name {
        object_list.retain(|obj| obj.name.to_lowercase().contains(&name.to_lowercase()));
    }

    // Apply pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);

    let total = object_list.len();
    object_list = object_list.into_iter().skip(offset).take(limit).collect();

    let response = serde_json::json!({
        "objects": object_list,
        "total": total,
        "offset": offset,
        "limit": limit
    });

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        response,
        "Objects retrieved successfully",
    )))
}

#[get("/objects/{id}")]
async fn get_object(store: ObjectStore, path: Path<u32>) -> Result<impl Responder> {
    let object_id = path.into_inner();
    let objects = store.lock().unwrap();

    match objects.get(&object_id) {
        Some(object) => Ok(HttpResponse::Ok().json(ApiResponse::success(object, "Object found"))),
        None => Ok(HttpResponse::NotFound().json(ErrorResponse::new("Object not found", 404))),
    }
}

#[post("/objects")]
async fn create_object(
    store: ObjectStore,
    object_data: Json<CreateObjectRequest>,
) -> Result<impl Responder> {
    let mut objects = store.lock().unwrap();
    let new_id = objects.len() as u32 + 1;

    // Basic validation
    if object_data.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse::new("Name cannot be empty", 400)));
    }

    if !object_data.email.contains('@') {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse::new("Invalid email format", 400)));
    }

    let new_object = Object {
        id: new_id,
        name: object_data.name.clone(),
        email: object_data.email.clone(),
        age: object_data.age,
    };

    objects.insert(new_id, new_object.clone());

    Ok(HttpResponse::Created().json(ApiResponse::success(
        new_object,
        "Object created successfully",
    )))
}

#[put("/objects/{id}")]
async fn update_object(
    store: ObjectStore,
    path: Path<u32>,
    object_data: Json<UpdateObjectRequest>,
) -> Result<impl Responder> {
    let object_id = path.into_inner();
    let mut objects = store.lock().unwrap();

    match objects.get_mut(&object_id) {
        Some(object) => {
            if let Some(name) = &object_data.name {
                if name.trim().is_empty() {
                    return Ok(HttpResponse::BadRequest()
                        .json(ErrorResponse::new("Name cannot be empty", 400)));
                }
                object.name = name.clone();
            }

            if let Some(email) = &object_data.email {
                if !email.contains('@') {
                    return Ok(HttpResponse::BadRequest()
                        .json(ErrorResponse::new("Invalid email format", 400)));
                }
                object.email = email.clone();
            }

            if let Some(age) = object_data.age {
                object.age = Some(age);
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                object.clone(),
                "Object updated successfully",
            )))
        }
        None => Ok(HttpResponse::NotFound().json(ErrorResponse::new("Object not found", 404))),
    }
}

#[patch("/objects/{id}")]
async fn patch_object(
    store: ObjectStore,
    path: Path<u32>,
    object_data: Json<UpdateObjectRequest>,
) -> Result<impl Responder> {
    let object_id = path.into_inner();
    let mut objects = store.lock().unwrap();

    match objects.get_mut(&object_id) {
        Some(object) => {
            // PATCH typically applies partial updates
            if let Some(name) = &object_data.name {
                if name.trim().is_empty() {
                    return Ok(HttpResponse::BadRequest()
                        .json(ErrorResponse::new("Name cannot be empty", 400)));
                }
                object.name = name.clone();
            }

            if let Some(email) = &object_data.email {
                if !email.contains('@') {
                    return Ok(HttpResponse::BadRequest()
                        .json(ErrorResponse::new("Invalid email format", 400)));
                }
                object.email = email.clone();
            }

            if let Some(age) = object_data.age {
                object.age = Some(age);
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(
                object.clone(),
                "Object updated successfully",
            )))
        }
        None => Ok(HttpResponse::NotFound().json(ErrorResponse::new("Object not found", 404))),
    }
}

#[delete("/objects/{id}")]
async fn delete_object(store: ObjectStore, path: Path<u32>) -> Result<impl Responder> {
    let object_id = path.into_inner();
    let mut objects = store.lock().unwrap();

    match objects.remove(&object_id) {
        Some(_) => Ok(
            HttpResponse::Ok().json(ApiResponse::success_no_data("Object deleted successfully"))
        ),
        None => Ok(HttpResponse::NotFound().json(ErrorResponse::new("Object not found", 404))),
    }
}

// Additional utility endpoints
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({"echoed": req_body}),
        "Message echoed",
    ))
}

#[get("/objects/{id}/profile")]
async fn get_object_profile(store: ObjectStore, path: Path<u32>) -> Result<impl Responder> {
    let object_id = path.into_inner();
    let objects = store.lock().unwrap();

    match objects.get(&object_id) {
        Some(object) => {
            let profile = serde_json::json!({
                "id": object.id,
                "name": object.name,
                "email": object.email,
                "age": object.age,
                "profile_url": format!("/objects/{}/profile", object.id),
                "created_at": "2024-01-01T00:00:00Z" // Mock timestamp
            });
            Ok(HttpResponse::Ok().json(ApiResponse::success(profile, "Profile retrieved")))
        }
        None => Ok(HttpResponse::NotFound().json(ErrorResponse::new("Object not found", 404))),
    }
}

#[get("/stats")]
async fn get_stats(store: ObjectStore) -> Result<impl Responder> {
    let objects = store.lock().unwrap();
    let total_objects = objects.len();
    let avg_age = objects
        .values()
        .filter_map(|obj| obj.age)
        .collect::<Vec<u32>>();

    let average_age = if !avg_age.is_empty() {
        avg_age.iter().sum::<u32>() as f64 / avg_age.len() as f64
    } else {
        0.0
    };

    let stats = serde_json::json!({
        "total_objects": total_objects,
        "objects_with_age": avg_age.len(),
        "average_age": average_age,
        "server_uptime": "unknown" // You could track this with a startup timestamp
    });

    Ok(HttpResponse::Ok().json(ApiResponse::success(stats, "Statistics retrieved")))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success_no_data("Hey there!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Initialize in-memory store
    let object_store = web::Data::new(std::sync::Mutex::new(HashMap::<u32, Object>::new()));

    println!("Starting server at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(object_store.clone())
            .wrap(Logger::default())
            // Basic routes
            .service(hello)
            .service(health_check)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            // Object CRUD routes
            .service(get_objects)
            .service(get_object)
            .service(create_object)
            .service(update_object)
            .service(patch_object)
            .service(delete_object)
            // Additional routes
            .service(get_object_profile)
            .service(get_stats)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
