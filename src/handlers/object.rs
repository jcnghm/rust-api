use actix_web::{get, post, put, patch, delete, web, HttpResponse, Result, ResponseError};
use crate::services::ObjectService;
use crate::models::object::*;
use crate::utils::ApiResponse;

#[get("/objects")]
pub async fn get_objects(
    service: web::Data<ObjectService>,
    query: web::Query<ObjectQuery>,
) -> Result<HttpResponse> {
    match service.get_objects(query.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(ApiResponse::success(response, "Objects retrieved successfully"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[get("/objects/{id}")]
pub async fn get_object(
    service: web::Data<ObjectService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let object_id = path.into_inner();
    
    match service.get_object(object_id).await {
        Ok(object) => Ok(HttpResponse::Ok().json(ApiResponse::success(object, "Object found"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[post("/objects")]
pub async fn create_object(
    service: web::Data<ObjectService>,
    req: web::Json<CreateObjectRequest>,
) -> Result<HttpResponse> {
    match service.create_object(req.into_inner()).await {
        Ok(object) => Ok(HttpResponse::Created().json(ApiResponse::success(object, "Object created successfully"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[put("/objects/{id}")]
pub async fn update_object(
    service: web::Data<ObjectService>,
    path: web::Path<i32>,
    req: web::Json<UpdateObjectRequest>,
) -> Result<HttpResponse> {
    let object_id = path.into_inner();
    
    match service.update_object(object_id, req.into_inner()).await {
        Ok(object) => Ok(HttpResponse::Ok().json(ApiResponse::success(object, "Object updated successfully"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[patch("/objects/{id}")]
pub async fn patch_object(
    service: web::Data<ObjectService>,
    path: web::Path<i32>,
    req: web::Json<UpdateObjectRequest>,
) -> Result<HttpResponse> {
    let object_id = path.into_inner();
    
    match service.update_object(object_id, req.into_inner()).await {
        Ok(object) => Ok(HttpResponse::Ok().json(ApiResponse::success(object, "Object updated successfully"))),
        Err(e) => Ok(e.error_response()),
    }
}

#[delete("/objects/{id}")]
pub async fn delete_object(
    service: web::Data<ObjectService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let object_id = path.into_inner();
    
    match service.delete_object(object_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::success_no_data("Object deleted successfully"))),
        Err(e) => Ok(e.error_response()),
    }
}