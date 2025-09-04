use std::path;

use crate::models::task::*;
use crate::services::TaskService;
use crate::utils::ApiResponse;
use actix_web::{HttpResponse, ResponseError, Result, delete, get, patch, post, put, web};

#[get("/")]
pub async fn get_tasks(
    service: web::Data<TaskService>,
    query: web::Query<TaskQuery>,
) -> Result<HttpResponse> {
    match service.get_tasks(query.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(
            response,
            "Tasks retrieved successfully",
        ))),
        Err(e) => Ok(e.error_response()),
    }
}

#[get("/{id}")]
pub async fn get_task(
    service: web::Data<TaskService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let task_id = path.into_inner();
    match service.get_task(task_id).await {
        Ok(task) => Ok(HttpResponse::Ok().json(ApiResponse::success(task, "Task found"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[post("/")]
pub async fn create_task(
    service: web::Data<TaskService>,
    req: web::Json<CreateTaskRequest>,
) -> Result<HttpResponse> {
    match service.create_task(req.into_inner()).await {
        Ok(task) => {
            Ok(HttpResponse::Created()
                .json(ApiResponse::success(task, "Task created successfully")))
        }
        Err(e) => Ok(e.error_response()),
    }
}

#[patch("/{id}")]
pub async fn update_task(
    service: web::Data<TaskService>,
    path: web::Path<i32>,
    req: web::Json<UpdateTaskRequest>,
) -> Result<HttpResponse> {
    let task_id = path.into_inner();
    match service.update_task(task_id, req.into_inner()).await {
        Ok(task) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(task, "Task updated successfully")))
        }
        Err(e) => Ok(e.error_response()),
    }
}

#[delete("/{id}")]
pub async fn delete_task(
    service: web::Data<TaskService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let task_id = path.into_inner();
    match service.delete_task(task_id).await {
        Ok(_) => Ok(
            HttpResponse::Ok().json(ApiResponse::<()>::success((), "Task deleted successfully"))
        ),
        Err(e) => Ok(e.error_response()),
    }
}

#[patch("/{task_id}/assign/{employee_id}")]
pub async fn assign_task(
    service: web::Data<TaskService>,
    path: web::Path<(i32, i32)>,
) -> Result<HttpResponse> {
    let (task_id, employee_id) = path.into_inner();
    match service.assign_task(task_id, employee_id).await {
        Ok(task) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(task, "Task assigned successfully")))
        }
        Err(e) => Ok(e.error_response()),
    }
}
