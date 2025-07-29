use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Object {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct CreateObjectRequest {
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateObjectRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub age: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ObjectQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub name: Option<String>,
}

impl Object {
    pub fn new(id: i32, name: String, email: String, age: Option<i32>) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            email,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}

impl CreateObjectRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        if let Some(age) = self.age {
            if age > 150 {
                return Err("Age must be realistic".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateObjectRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err("Name cannot be empty".to_string());
            }
        }

        if let Some(email) = &self.email {
            if !email.contains('@') {
                return Err("Invalid email format".to_string());
            }
        }

        if let Some(age) = self.age {
            if age > 150 {
                return Err("Age must be realistic".to_string());
            }
        }

        Ok(())
    }
}
