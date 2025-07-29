use crate::models::object::*;
use crate::repositories::ObjectRepository;
use crate::errors::ApiError;

#[derive(Clone)]
pub struct ObjectService {
    repository: ObjectRepository,
}

impl ObjectService {
    pub fn new(repository: ObjectRepository) -> Self {
        Self { repository }
    }

    pub async fn create_object(&self, req: CreateObjectRequest) -> Result<Object, ApiError> {
        // Validate input
        req.validate().map_err(ApiError::ValidationError)?;

        // Business logic could go here (e.g., duplicate email check, etc.)
        
        self.repository.create(req).await
    }

    pub async fn get_object(&self, id: i32) -> Result<Object, ApiError> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_objects(&self, query: ObjectQuery) -> Result<serde_json::Value, ApiError> {
        let (objects, total) = self.repository.find_all(query.clone()).await?;
        
        Ok(serde_json::json!({
            "objects": objects,
            "total": total,
            "offset": query.offset.unwrap_or(0),
            "limit": query.limit.unwrap_or(10)
        }))
    }

    pub async fn update_object(&self, id: i32, req: UpdateObjectRequest) -> Result<Object, ApiError> {
        // Validate input
        req.validate().map_err(ApiError::ValidationError)?;

        self.repository.update(id, req).await
    }

    pub async fn delete_object(&self, id: i32) -> Result<(), ApiError> {
        self.repository.delete(id).await
    }

    pub async fn get_object_profile(&self, id: i32) -> Result<serde_json::Value, ApiError> {
        let object = self.repository.find_by_id(id).await?;
        
        Ok(serde_json::json!({
            "id": object.id,
            "name": object.name,
            "email": object.email,
            "age": object.age,
            "profile_url": format!("/objects/{}/profile", object.id),
            "created_at": object.created_at,
            "updated_at": object.updated_at
        }))
    }

    pub async fn get_stats(&self) -> Result<serde_json::Value, ApiError> {
        self.repository.get_stats().await
    }
}