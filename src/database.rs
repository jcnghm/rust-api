use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::config::AppConfig;
use std::path::Path;

pub async fn create_pool(config: &AppConfig) -> Result<SqlitePool, sqlx::Error> {
    // Ensure the database directory exists
    if let Some(db_path) = extract_db_path(&config.database_url) {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                sqlx::Error::Configuration(format!("Failed to create database directory: {}", e).into())
            })?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

fn extract_db_path(database_url: &str) -> Option<&str> {
    if database_url.starts_with("sqlite://") {
        Some(&database_url[9..]) // Remove "sqlite://" prefix
    } else if database_url.starts_with("sqlite:") {
        Some(&database_url[7..]) // Remove "sqlite:" prefix
    } else {
        None
    }
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create objects table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS objects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create index on email for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_email ON objects(email)
        "#
    )
    .execute(pool)
    .await?;

    // Create index on created_at for faster sorting
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_created_at ON objects(created_at)
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
} 