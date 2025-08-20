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
            SELECT id, external_id, first_name, last_name, store_id, email
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
            email: row.get("email"),
            store_id: row.get("store_id"),
        };

        Ok(employee)
    }

    pub async fn find_all(&self, query: EmployeeQuery) -> Result<(Vec<Employee>, usize), ApiError> {
        let mut sql = String::from(
            "SELECT id, external_id, first_name, last_name, store_id, email, manager_id FROM employees",
        );
        let mut count_sql = String::from("SELECT COUNT(*) FROM employees");
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(store_id) = query.store_id {
            conditions.push("store_id = ?");
            params.push(store_id.to_string());
        }

        if let Some(first_name) = &query.first_name {
            conditions.push("first_name LIKE ?");
            params.push(format!("%{}%", first_name));
        }

        if let Some(last_name) = &query.last_name {
            conditions.push("last_name LIKE ?");
            params.push(format!("%{}%", last_name));
        }

        if let Some(email) = &query.email {
            sql.push_str(" AND email = ?");
            params.push(email.clone());
        }

        if let Some(external_id) = &query.external_id {
            sql.push_str(" AND external_id = ?");
            params.push(external_id.clone());
        }

        if !conditions.is_empty() {
            let where_clause = conditions.join(" AND ");
            sql.push_str(&format!(" WHERE {}", where_clause));
            count_sql.push_str(&format!(" WHERE {}", where_clause));
        }

        sql.push_str(" ORDER BY last_name ASC, first_name ASC");

        let limit = query.limit.unwrap_or(10);
        let offset = query.offset.unwrap_or(0);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

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
                email: row.get("email"),
            })
            .collect();

        Ok((employees, total as usize))
    }

    pub async fn create_bulk(
        &self,
        employees: Vec<CreateEmployee>,
    ) -> Result<Vec<Employee>, ApiError> {
        if employees.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Transaction error: {}", e)))?;

        let mut created_ids = Vec::new();

        for employee in employees {
            let result = sqlx::query(
                r#"
                INSERT INTO employees (external_id, first_name, last_name, store_id, email)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(&employee.external_id)
            .bind(&employee.first_name)
            .bind(&employee.last_name)
            .bind(employee.store_id)
            .bind(&employee.email)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

            created_ids.push(result.last_insert_rowid() as i32);
        }
        tx.commit().await.map_err(|e| {
            ApiError::InternalServerError(format!("Transaction commit error: {}", e))
        })?;

        let mut created_employees = Vec::new();
        for id in created_ids {
            created_employees.push(self.find_by_id(id).await?);
        }

        Ok(created_employees)
    }
}
