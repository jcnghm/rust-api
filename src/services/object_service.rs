use crate::errors::ApiError;
use crate::models::object::*;
use crate::repositories::ObjectRepository;

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

    pub async fn update_object(
        &self,
        id: i32,
        req: UpdateObjectRequest,
    ) -> Result<Object, ApiError> {
        // Validate input
        req.validate().map_err(ApiError::ValidationError)?;

        self.repository.update(id, req).await
    }

    pub async fn delete_object(&self, id: i32) -> Result<(), ApiError> {
        self.repository.delete(id).await
    }
}
