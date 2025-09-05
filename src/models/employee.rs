use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Employee {
    pub id: i32,
    pub external_id: String,
    pub first_name: String,
    pub last_name: String,
    pub store_id: Option<i32>,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmployeeQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub store_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CreateEmployee {
    pub external_id: String,
    pub first_name: String,
    pub last_name: String,
    pub store_id: Option<i32>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CreateEmployeesRequest {
    pub employees: Vec<CreateEmployee>,
}

impl CreateEmployeesRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.employees.is_empty() {
            return Err("Employee list cannot be empty".to_string());
        }
        for employee in &self.employees {
            if employee.external_id.trim().is_empty() {
                return Err("External ID cannot be empty".to_string());
            }
            if employee.first_name.trim().is_empty() {
                return Err("First name cannot be empty".to_string());
            }
            if employee.last_name.trim().is_empty() {
                return Err("Last name cannot be empty".to_string());
            }
            if let Some(email) = &employee.email {
                if !email.contains('@') {
                    return Err(format!("Invalid email format: {}", email));
                }
            }
        }
        Ok(())
    }
}

/**
 * Unit tests for Employee model and related structs
 *
 *
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_new() {
        let employee = Employee {
            id: 1,
            external_id: "ext123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            store_id: Some(10),
            email: Some("test@test.com".to_string()),
        };

        assert_eq!(employee.id, 1);
        assert_eq!(employee.external_id, "ext123");
        assert_eq!(employee.first_name, "John");
        assert_eq!(employee.last_name, "Doe");
        assert_eq!(employee.store_id, Some(10));
        assert_eq!(employee.email, Some("test@test.com".to_string()));
    }

    #[test]
    fn test_create_employee_request_validate_success() {
        let request = CreateEmployeesRequest {
            employees: vec![CreateEmployee {
                external_id: "ext123".to_string(),
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                store_id: Some(10),
                email: Some("test@test.com".to_string()),
            }],
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_employee_request_validate_empty_list() {
        let request = CreateEmployeesRequest { employees: vec![] };
        assert_eq!(
            request.validate().unwrap_err(),
            "Employee list cannot be empty"
        );
    }
}
