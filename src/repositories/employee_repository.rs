use crate::errors::ApiError;
use crate::models::employee::*;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct EmployeeRepository {
    pool: SqlitePool,
}

impl EmployeeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Employee, ApiError> {
        let row = sqlx::query(
            r#"
            SELECT id, external_id, first_name, last_name, store_id
            FROM employees
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Employee not found".to_string()))?;

        let employee = Employee {
            id: row.get("id"),
            external_id: row.get("external_id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            store_id: row.get("store_id"),
        };

        Ok(employee)
    }

    pub async fn find_all(&self, query: EmployeeQuery) -> Result<(Vec<Employee>, usize), ApiError> {
        // Build the base query
        let mut sql =
            String::from("SELECT id, external_id, first_name, last_name, store_id FROM employees");
        let mut count_sql = String::from("SELECT COUNT(*) FROM employees");
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        // Add store_id filter if provided
        if let Some(store_id) = query.store_id {
            conditions.push("store_id = ?");
            params.push(store_id.to_string());
        }

        // Add first_name filter if provided
        if let Some(first_name) = &query.first_name {
            conditions.push("first_name LIKE ?");
            params.push(format!("%{}%", first_name));
        }

        // Add last_name filter if provided
        if let Some(last_name) = &query.last_name {
            conditions.push("last_name LIKE ?");
            params.push(format!("%{}%", last_name));
        }

        // Add WHERE clause if conditions exist
        if !conditions.is_empty() {
            let where_clause = conditions.join(" AND ");
            sql.push_str(&format!(" WHERE {}", where_clause));
            count_sql.push_str(&format!(" WHERE {}", where_clause));
        }

        // Add ORDER BY
        sql.push_str(" ORDER BY last_name ASC, first_name ASC");

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

        // Get employees
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

        let employees: Vec<Employee> = rows
            .into_iter()
            .map(|row| Employee {
                id: row.get("id"),
                external_id: row.get("external_id"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                store_id: row.get("store_id"),
            })
            .collect();

        Ok((employees, total as usize))
    }
}
