use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub priority_level: Option<PriorityLevel>,
    pub status: Option<TaskStatus>,
    pub assigned_to: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority_level: Option<PriorityLevel>,
    pub status: Option<TaskStatus>,
    pub assigned_to: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority_level: Option<PriorityLevel>,
    pub status: Option<TaskStatus>,
    pub assigned_to: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TaskQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub id: Option<i32>,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority_level: Option<PriorityLevel>,
    pub assigned_to: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskStatus::ToDo => write!(f, "To Do"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
        }
    }
}

impl fmt::Display for PriorityLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PriorityLevel::Low => write!(f, "Low"),
            PriorityLevel::Medium => write!(f, "Medium"),
            PriorityLevel::High => write!(f, "High"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")] // Store as TEXT in database
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
}

impl Task {
    pub fn new(
        id: i32,
        title: String,
        description: Option<String>,
        priority_level: Option<PriorityLevel>,
        status: Option<TaskStatus>,
        assigned_to: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            priority_level: None,
            status: Some(TaskStatus::ToDo),
            assigned_to,
            completed_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl CreateTaskRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        Ok(())
    }
}

impl UpdateTaskRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
        }

        Ok(())
    }
}
