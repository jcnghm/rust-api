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
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_email ON objects(email)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_objects_created_at ON objects(created_at)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

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
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

pub async fn create_test_pool_with_employees() -> SqlitePool {
    let pool = create_test_pool().await;
    
    // Create employees table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            store_id INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create indexes for employees
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_store_id ON employees(store_id)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_external_id ON employees(external_id)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_employees_name ON employees(last_name, first_name)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

// Helper function to seed test employee data
pub async fn seed_test_employees(pool: &SqlitePool) {
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

    for (external_id, first_name, last_name, store_id) in employees {
        sqlx::query(
            r#"
            INSERT INTO employees (external_id, first_name, last_name, store_id)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(external_id)
        .bind(first_name)
        .bind(last_name)
        .bind(store_id)
        .execute(pool)
        .await
        .unwrap();
    }
}