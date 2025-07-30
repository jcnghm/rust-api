use crate::errors::ApiError;
use crate::models::employee::*;
use crate::repositories::EmployeeRepository;

#[derive(Clone)]
pub struct EmployeeService {
    repository: EmployeeRepository,
}

impl EmployeeService {
    pub fn new(repository: EmployeeRepository) -> Self {
        Self { repository }
    }

    pub async fn get_employee(&self, id: i32) -> Result<Employee, ApiError> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_employees(&self, query: EmployeeQuery) -> Result<serde_json::Value, ApiError> {
        let (employees, total) = self.repository.find_all(query.clone()).await?;

        Ok(serde_json::json!({
            "employees": employees,
            "total": total,
            "offset": query.offset.unwrap_or(0),
            "limit": query.limit.unwrap_or(10)
        }))
    }

    pub async fn get_employees_by_store(
        &self,
        store_id: i32,
        query: EmployeeQuery,
    ) -> Result<serde_json::Value, ApiError> {
        let mut store_query = query.clone();
        store_query.store_id = Some(store_id);

        self.get_employees(store_query).await
    }
}
