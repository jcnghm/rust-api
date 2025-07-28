use crate::models::object::*;
use crate::errors::ApiError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ObjectStore = Arc<Mutex<HashMap<u32, Object>>>;

#[derive(Clone)]
pub struct ObjectRepository {
    store: ObjectStore,
}

impl ObjectRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create(&self, req: CreateObjectRequest) -> Result<Object, ApiError> {
        let mut objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        let new_id = objects.len() as u32 + 1;
        let new_object = Object::new(new_id, req.name, req.email, req.age);
        
        objects.insert(new_id, new_object.clone());
        Ok(new_object)
    }

    pub async fn find_by_id(&self, id: u32) -> Result<Object, ApiError> {
        let objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        objects.get(&id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))
    }

    pub async fn find_all(&self, query: ObjectQuery) -> Result<(Vec<Object>, usize), ApiError> {
        let objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        let mut object_list: Vec<Object> = objects.values().cloned().collect();

        // Filter by name if provided
        if let Some(name) = &query.name {
            object_list.retain(|obj| obj.name.to_lowercase().contains(&name.to_lowercase()));
        }

        let total = object_list.len();

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(10);
        
        object_list = object_list.into_iter().skip(offset).take(limit).collect();

        Ok((object_list, total))
    }

    pub async fn update(&self, id: u32, req: UpdateObjectRequest) -> Result<Object, ApiError> {
        let mut objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        let object = objects.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

        if let Some(name) = req.name {
            object.name = name;
        }
        if let Some(email) = req.email {
            object.email = email;
        }
        if let Some(age) = req.age {
            object.age = Some(age);
        }

        Ok(object.clone())
    }

    pub async fn delete(&self, id: u32) -> Result<(), ApiError> {
        let mut objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        objects.remove(&id)
            .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<serde_json::Value, ApiError> {
        let objects = self.store.lock().map_err(|_| {
            ApiError::InternalServerError("Failed to acquire lock".to_string())
        })?;

        let total_objects = objects.len();
        let ages: Vec<u32> = objects.values()
            .filter_map(|obj| obj.age)
            .collect();

        let average_age = if !ages.is_empty() {
            ages.iter().sum::<u32>() as f64 / ages.len() as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "total_objects": total_objects,
            "objects_with_age": ages.len(),
            "average_age": average_age,
            "server_uptime": "unknown"
        }))
    }
}