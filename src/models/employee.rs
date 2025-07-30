use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Employee {
    pub id: i32,
    pub external_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub store_id: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmployeeQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub store_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
