use crate::errors::ApiError;
use crate::models::object::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct ObjectRepository {
    pool: SqlitePool,
}

impl ObjectRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateObjectRequest) -> Result<Object, ApiError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO objects (name, email, age, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.name)
        .bind(&req.email)
        .bind(req.age)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        let id = result.last_insert_rowid() as i32;
        Ok(Object::new(id, req.name, req.email, req.age))
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Object, ApiError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, email, age, created_at, updated_at
            FROM objects
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

        let object = Object {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            age: row.get("age"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(object)
    }

    pub async fn find_all(&self, query: ObjectQuery) -> Result<(Vec<Object>, usize), ApiError> {
        // Build the base query
        let mut sql =
            String::from("SELECT id, name, email, age, created_at, updated_at FROM objects");
        let mut count_sql = String::from("SELECT COUNT(*) FROM objects");
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        // Add name filter if provided
        if let Some(name) = &query.name {
            conditions.push("name LIKE ?");
            params.push(format!("%{}%", name));
        }

        // Add WHERE clause if conditions exist
        if !conditions.is_empty() {
            let where_clause = conditions.join(" AND ");
            sql.push_str(&format!(" WHERE {}", where_clause));
            count_sql.push_str(&format!(" WHERE {}", where_clause));
        }

        // Add ORDER BY
        sql.push_str(" ORDER BY created_at DESC");

        // Add LIMIT and OFFSET
        let limit = query.limit.unwrap_or(10);
        let offset = query.offset.unwrap_or(0);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // Get total count
        let total: i64 = if params.is_empty() {
            sqlx::query(&count_sql)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
                .get(0)
        } else {
            let mut query = sqlx::query(&count_sql);
            for param in &params {
                query = query.bind(param);
            }
            query
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
                .get(0)
        };

        // Get objects
        let rows = if params.is_empty() {
            sqlx::query(&sql)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        } else {
            let mut query = sqlx::query(&sql);
            for param in &params {
                query = query.bind(param);
            }
            query
                .fetch_all(&self.pool)
                .await
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        };

        let objects: Vec<Object> = rows
            .into_iter()
            .map(|row| Object {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                age: row.get("age"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok((objects, total as usize))
    }

    pub async fn update(&self, id: i32, req: UpdateObjectRequest) -> Result<Object, ApiError> {
        // First check if object exists
        self.find_by_id(id).await?;

        let mut sql = String::from("UPDATE objects SET updated_at = ?");
        let mut params: Vec<String> = Vec::new();
        let now = Utc::now();

        if let Some(name) = &req.name {
            sql.push_str(", name = ?");
            params.push(name.clone());
        }
        if let Some(email) = &req.email {
            sql.push_str(", email = ?");
            params.push(email.clone());
        }
        if let Some(age) = req.age {
            sql.push_str(", age = ?");
            params.push(age.to_string());
        }

        sql.push_str(" WHERE id = ?");

        // Execute update
        let mut query = sqlx::query(&sql).bind(now);
        for param in &params {
            query = query.bind(param);
        }
        query = query.bind(id);

        query
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        // Return updated object
        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), ApiError> {
        let result = sqlx::query("DELETE FROM objects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound("Object not found".to_string()));
        }

        Ok(())
    }
}
