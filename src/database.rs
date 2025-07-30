use crate::config::AppConfig;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

pub async fn create_pool(config: &AppConfig) -> Result<SqlitePool, sqlx::Error> {
    // Ensure the database directory exists
    if let Some(db_path) = extract_db_path(&config.database_url) {
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                sqlx::Error::Configuration(
                    format!("Failed to create database directory: {}", e).into(),
                )
            })?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    // Seed test data
    seed_employees(&pool).await?;

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
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on email for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_email ON objects(email)
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on created_at for faster sorting
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_created_at ON objects(created_at)
        "#,
    )
    .execute(pool)
    .await?;

    // Create employees table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            store_id INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on store_id for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_store_id ON employees(store_id)
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on external_id for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_external_id ON employees(external_id)
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on last_name, first_name for faster sorting
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_name ON employees(last_name, first_name)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn seed_employees(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if employees table already has data
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM employees")
        .fetch_one(pool)
        .await?
        .get(0);

    // Only seed if table is empty
    if count == 0 {
        println!("Seeding employee test data...");

        let employees = vec![
            ("EMP001", "John", "Smith", 1),
            ("EMP002", "Sarah", "Johnson", 1),
            ("EMP003", "Michael", "Brown", 1),
            ("EMP004", "Emily", "Davis", 2),
            ("EMP005", "David", "Wilson", 2),
            ("EMP006", "Lisa", "Anderson", 2),
            ("EMP007", "James", "Taylor", 3),
            ("EMP008", "Jennifer", "Martinez", 3),
            ("EMP009", "Robert", "Garcia", 3),
            ("EMP010", "Amanda", "Rodriguez", 1),
            ("EMP011", "Christopher", "Lee", 2),
            ("EMP012", "Jessica", "White", 3),
        ];

        let employee_count = employees.len();

        for (external_id, first_name, last_name, store_id) in employees {
            sqlx::query(
                r#"
                INSERT INTO employees (external_id, first_name, last_name, store_id)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(external_id)
            .bind(first_name)
            .bind(last_name)
            .bind(store_id)
            .execute(pool)
            .await?;
        }

        println!("✅ Seeded {} employees", employee_count);
    }

    Ok(())
}
