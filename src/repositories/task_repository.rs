use crate::errors::ApiError;
use crate::models::task::*;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct TaskRepository {
    pool: SqlitePool,
}

impl TaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateTaskRequest) -> Result<Task, ApiError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO tasks (title, description, priority_level, status, assigned_to, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.priority_level.map(|p| p.to_string()))
        .bind(req.status.map(|s| s.to_string()).unwrap_or("ToDo".to_string()))
        .bind(req.assigned_to)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        let id = result.last_insert_rowid() as i32;
        Ok(Task::new(
            id,
            req.title,
            req.description,
            req.priority_level,
            Some(req.status.clone().unwrap_or(TaskStatus::ToDo)),
            req.assigned_to,
        ))
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Task, ApiError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, description, priority_level, status, assigned_to, completed_at, created_at, updated_at, deleted_at
            FROM tasks
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Task not found".to_string()))?;

        let task = Task {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            priority_level: row
                .get::<Option<String>, _>("priority_level")
                .and_then(|s| match s.as_str() {
                    "Low" => Some(PriorityLevel::Low),
                    "Medium" => Some(PriorityLevel::Medium),
                    "High" => Some(PriorityLevel::High),
                    _ => None,
                }),
            status: row
                .get::<Option<String>, _>("status")
                .and_then(|s| match s.as_str() {
                    "ToDo" => Some(TaskStatus::ToDo),
                    "InProgress" => Some(TaskStatus::InProgress),
                    "Done" => Some(TaskStatus::Done),
                    _ => None,
                }),
            assigned_to: row.get("assigned_to"),
            completed_at: row.get("completed_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        };

        Ok(task)
    }

    pub async fn find_all(&self, query: TaskQuery) -> Result<(Vec<Task>, usize), ApiError> {
        let mut sql = String::from(
            "SELECT id, title, description, priority_level, status, assigned_to, completed_at, created_at, updated_at, deleted_at FROM tasks",
        );
        let mut count_sql = String::from("SELECT COUNT(*) FROM tasks");
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(title) = &query.title {
            conditions.push("title LIKE ?");
            params.push(format!("%{}%", title));
        }

        if let Some(status) = &query.status {
            conditions.push("status = ?");
            params.push(status.to_string());
        }

        if let Some(priority_level) = &query.priority_level {
            conditions.push("priority_level = ?");
            params.push(priority_level.to_string());
        }

        if let Some(assigned_to) = query.assigned_to {
            conditions.push("assigned_to = ?");
            params.push(assigned_to.to_string());
        }

        if !conditions.is_empty() {
            let where_clause = conditions.join(" AND ");
            sql.push_str(&format!(" WHERE {}", where_clause));
            count_sql.push_str(&format!(" WHERE {}", where_clause));
        }

        if let Some(sort_by) = &query.sort_by {
            let direction = query.sort_direction.as_deref().unwrap_or("asc");
            sql.push_str(&format!(" ORDER BY {} {}", sort_by, direction));
        } else {
            sql.push_str(" ORDER BY created_at DESC");
        }

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            sql.push_str(" LIMIT 10");
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        } else {
            sql.push_str(" OFFSET 0");
        }

        let mut query = sqlx::query_as::<_, Task>(&sql);
        for param in &params {
            query = query.bind(param);
        }

        let tasks = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for param in &params {
            count_query = count_query.bind(param);
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        Ok((tasks, total as usize))
    }

    pub async fn update(&self, id: i32, req: UpdateTaskRequest) -> Result<Task, ApiError> {
        let existing_task = self.find_by_id(id).await?;
        let updated_title = req.title.unwrap_or(existing_task.title);
        let updated_description = req.description.or(existing_task.description);
        let updated_priority_level = req.priority_level.or(existing_task.priority_level);
        let updated_status = req.status.or(existing_task.status);
        let updated_assigned_to = req.assigned_to.or(existing_task.assigned_to);
        let updated_completed_at = if let Some(TaskStatus::Done) = updated_status {
            Some(Utc::now())
        } else {
            None
        };
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE tasks
            SET title = ?, description = ?, priority_level = ?, status = ?, assigned_to = ?, completed_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )   .bind(&updated_title)
            .bind(&updated_description)
            .bind(updated_priority_level.map(|p| p.to_string()))
            .bind(updated_status.map(|s| s.to_string()))
            .bind(updated_assigned_to)
            .bind(updated_completed_at)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
        self.find_by_id(id).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), ApiError> {
        self.find_by_id(id).await?;
        let now = Utc::now();
        sqlx::query("UPDATE tasks SET deleted_at = ? WHERE id = ?")
            .bind(id)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        Ok(())
    }

    pub async fn assign(&self, task_id: i32, employee_id: i32) -> Result<Task, ApiError> {
        sqlx::query("UPDATE tasks SET assigned_to = ?, updated_at = ? WHERE id = ?")
            .bind(employee_id)
            .bind(Utc::now())
            .bind(task_id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

        self.find_by_id(task_id).await
    }
}
