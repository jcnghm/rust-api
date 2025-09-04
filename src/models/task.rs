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
            priority_level: Some(priority_level.unwrap_or(PriorityLevel::Medium)),
            status: status.or(Some(TaskStatus::ToDo)),
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

/**
 * Unit tests for Task model and related structs
 *
 *
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let task = Task::new(
            1,
            "Test Task".to_string(),
            Some("Test description".to_string()),
            Some(PriorityLevel::High),
            Some(TaskStatus::InProgress),
            Some(42),
        );

        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, Some("Test description".to_string()));
        assert_eq!(task.priority_level, Some(PriorityLevel::High));
        assert_eq!(task.status, Some(TaskStatus::InProgress));
        assert_eq!(task.assigned_to, Some(42));
        assert!(task.completed_at.is_none());
        assert!(task.deleted_at.is_none());
    }

    #[test]
    fn test_task_new_with_defaults() {
        let task = Task::new(1, "Test Task".to_string(), None, None, None, None);

        assert_eq!(task.status, Some(TaskStatus::ToDo));
        assert!(task.description.is_none());
        assert!(task.priority_level.is_some());
        assert!(task.assigned_to.is_none());
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::ToDo.to_string(), "To Do");
        assert_eq!(TaskStatus::InProgress.to_string(), "In Progress");
        assert_eq!(TaskStatus::Done.to_string(), "Done");
    }

    #[test]
    fn test_priority_level_display() {
        assert_eq!(PriorityLevel::Low.to_string(), "Low");
        assert_eq!(PriorityLevel::Medium.to_string(), "Medium");
        assert_eq!(PriorityLevel::High.to_string(), "High");
    }

    #[test]
    fn test_create_task_request_validate_success() {
        let request = CreateTaskRequest {
            title: "Valid Title".to_string(),
            description: Some("Valid description".to_string()),
            priority_level: Some(PriorityLevel::Medium),
            status: Some(TaskStatus::ToDo),
            assigned_to: Some(1),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_task_request_validate_empty_title() {
        let request = CreateTaskRequest {
            title: "".to_string(),
            description: None,
            priority_level: None,
            status: None,
            assigned_to: None,
        };

        assert!(request.validate().is_err());
        assert_eq!(request.validate().unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_create_task_request_validate_whitespace_title() {
        let request = CreateTaskRequest {
            title: "   ".to_string(),
            description: None,
            priority_level: None,
            status: None,
            assigned_to: None,
        };

        assert!(request.validate().is_err());
        assert_eq!(request.validate().unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_update_task_request_validate_success() {
        let request = UpdateTaskRequest {
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            priority_level: Some(PriorityLevel::High),
            status: Some(TaskStatus::Done),
            assigned_to: Some(2),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_task_request_validate_all_none() {
        let request = UpdateTaskRequest {
            title: None,
            description: None,
            priority_level: None,
            status: None,
            assigned_to: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_update_task_request_validate_empty_title() {
        let request = UpdateTaskRequest {
            title: Some("".to_string()),
            description: None,
            priority_level: None,
            status: None,
            assigned_to: None,
        };

        assert!(request.validate().is_err());
        assert_eq!(request.validate().unwrap_err(), "Title cannot be empty");
    }

    #[test]
    fn test_task_status_equality() {
        assert_eq!(TaskStatus::ToDo, TaskStatus::ToDo);
        assert_ne!(TaskStatus::ToDo, TaskStatus::InProgress);
    }

    #[test]
    fn test_priority_level_equality() {
        assert_eq!(PriorityLevel::High, PriorityLevel::High);
        assert_ne!(PriorityLevel::Low, PriorityLevel::High);
    }

    #[test]
    fn test_task_clone() {
        let task = Task::new(
            1,
            "Test Task".to_string(),
            None,
            Some(PriorityLevel::Medium),
            None,
            None,
        );

        let cloned_task = task.clone();
        assert_eq!(task.id, cloned_task.id);
        assert_eq!(task.title, cloned_task.title);
        assert_eq!(task.priority_level, cloned_task.priority_level);
    }

    #[test]
    fn test_serialization() {
        let task = Task::new(
            1,
            "Test Task".to_string(),
            Some("Description".to_string()),
            Some(PriorityLevel::High),
            Some(TaskStatus::InProgress),
            Some(42),
        );

        let _json = serde_json::to_string(&task).expect("Should serialize successfully");
    }
}
