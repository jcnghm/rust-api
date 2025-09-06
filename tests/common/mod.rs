use sqlx::SqlitePool;

pub async fn create_test_pool() -> SqlitePool {
    let test_db_url = "sqlite::memory:";
    let pool = SqlitePool::connect(test_db_url).await.unwrap();

    // Run migrations for objects table
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
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_email ON objects(email)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_created_at ON objects(created_at)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add employees table (needed for tasks foreign key)
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
    .execute(&pool)
    .await
    .unwrap();

    // Create employees indexes
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_store_id ON employees(store_id)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_external_id ON employees(external_id)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_name ON employees(last_name, first_name)
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Run migrations for tasks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            priority_level TEXT,
            status TEXT NOT NULL,
            assigned_to INTEGER,
            completed_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (assigned_to) REFERENCES employees(id) ON DELETE SET NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
