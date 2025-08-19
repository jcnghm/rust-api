use crate::models::employee::*;
use crate::services::EmployeeService;
use crate::utils::ApiResponse;
use actix_web::{HttpResponse, ResponseError, Result, get, web, post};

#[get("/")]
pub async fn get_employees(
    service: web::Data<EmployeeService>,
    query: web::Query<EmployeeQuery>,
) -> Result<HttpResponse> {
    match service.get_employees(query.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            response,
            "Employees retrieved successfully",
        ))),
        Err(e) => Ok(e.error_response()),
    }
}

#[get("/{id}")]
pub async fn get_employee(
    service: web::Data<EmployeeService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let employee_id = path.into_inner();

    match service.get_employee(employee_id).await {
        Ok(employee) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(employee, "Employee found")))
        }
        Err(e) => Ok(e.error_response()),
    }
}

#[get("/stores/{store_id}")]
pub async fn get_employees_by_store(
    service: web::Data<EmployeeService>,
    path: web::Path<i32>,
    query: web::Query<EmployeeQuery>,
) -> Result<HttpResponse> {
    let store_id = path.into_inner();

    match service
        .get_employees_by_store(store_id, query.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            response,
            "Store employees retrieved successfully",
        ))),
        Err(e) => Ok(e.error_response()),
    }
}

#[post("/")]
pub async fn create_employees(
    service: web::Data<EmployeeService>,
    request: web::Json<CreateEmployeesRequest>,
) -> Result<HttpResponse> {
    print!("test");
    match service.create_employees(request.employees.clone()).await {
        Ok(created_employees) => Ok(HttpResponse::Created().json(ApiResponse::success(
            serde_json::json!({
                "employees": created_employees,
                "count": created_employees.len()
            }),
            "Employees created successfully",
        ))),
        Err(e) => Ok(e.error_response()),
    }
}
