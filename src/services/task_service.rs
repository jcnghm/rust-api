use crate::errors::ApiError;
use crate::models::task::*;
use crate::repositories::TaskRepository;

#[derive(Clone)]
pub struct TaskService {
    repository: TaskRepository,
}

impl TaskService {
    pub fn new(repository: TaskRepository) -> Self {
        Self { repository }
    }

    pub async fn create_task(&self, req: CreateTaskRequest) -> Result<Task, ApiError> {
        req.validate().map_err(ApiError::ValidationError)?;

        self.repository.create(req).await
    }

    pub async fn get_task(&self, id: i32) -> Result<Task, ApiError> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_tasks(&self, query: TaskQuery) -> Result<serde_json::Value, ApiError> {
        let (tasks, total) = self.repository.find_all(query.clone()).await?;

        Ok(serde_json::json!({
            "tasks": tasks,
            "total": total,
            "offset": query.offset.unwrap_or(0),
            "limit": query.limit.unwrap_or(10)
        }))
    }
    pub async fn update_task(&self, id: i32, req: UpdateTaskRequest) -> Result<Task, ApiError> {
        req.validate().map_err(ApiError::ValidationError)?;

        self.repository.update(id, req).await
    }
    pub async fn delete_task(&self, id: i32) -> Result<(), ApiError> {
        self.repository.delete(id).await
    }

    pub async fn assign_task(&self, task_id: i32, employee_id: i32) -> Result<Task, ApiError> {
        self.repository.assign(task_id, employee_id).await
    }
}
