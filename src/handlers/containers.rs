use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::AppError;
use crate::models::{
    Container, ContainerWithItemCount, CreateContainerRequest, UpdateContainerRequest,
};


#[derive(Debug, Deserialize)]
pub struct ListContainersQuery {
    pub location: Option<String>,
    pub include_disposed: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateContainerResponse {
    pub container: Container,
}

#[derive(Debug, Serialize)]
pub struct GetContainerResponse {
    pub container: Container,
}

#[derive(Debug, Serialize)]
pub struct ListContainersResponse {
    pub containers: Vec<ContainerWithItemCount>,
}

#[derive(Debug, Serialize)]
pub struct UpdateContainerResponse {
    pub container: Container,
}

pub async fn create_container(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Json(request): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<CreateContainerResponse>), StatusCode> {
    if request.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match container_service.create_container(request).await {
        Ok(container) => Ok((
            StatusCode::CREATED,
            Json(CreateContainerResponse { container }),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_container(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetContainerResponse>, StatusCode> {
    match container_service.get_container(&id).await {
        Ok(container) => Ok(Json(GetContainerResponse { container })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_containers(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Query(query): Query<ListContainersQuery>,
) -> Result<Json<ListContainersResponse>, StatusCode> {
    let location_filter = query.location.as_deref();
    let include_disposed = query.include_disposed.unwrap_or(false);
    let search = query.search.as_deref();
    let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");

    match container_service
        .list_containers(
            location_filter,
            include_disposed,
            search,
            sort_by,
            sort_order,
        )
        .await
    {
        Ok(containers) => Ok(Json(ListContainersResponse { containers })),
        Err(e) => {
           tracing::error!("Failed to list containers: {:?}", e);
           Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_container(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateContainerRequest>,
) -> Result<Json<UpdateContainerResponse>, StatusCode> {
    if request.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match container_service.update_container(&id, request).await {
        Ok(container) => Ok(Json(UpdateContainerResponse { container })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_container(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match container_service.delete_container(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Serialize)]
pub struct CheckContainerIdResponse {
    pub exists: bool,
}

pub async fn check_container_id(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Path(id): Path<String>,
) -> Result<Json<CheckContainerIdResponse>, StatusCode> {
    match container_service.check_container_id_exists(&id).await {
        Ok(exists) => Ok(Json(CheckContainerIdResponse { exists })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Serialize)]
pub struct GetContainersByLocationResponse {
    pub containers: Vec<Container>,
}

pub async fn get_containers_by_location(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Path(location): Path<String>,
) -> Result<Json<GetContainersByLocationResponse>, StatusCode> {
    match container_service.get_containers_by_location(&location).await {
        Ok(containers) => Ok(Json(GetContainersByLocationResponse { containers })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteContainersRequest {
    pub ids: Vec<String>,
}

pub async fn bulk_delete_containers(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Json(request): Json<BulkDeleteContainersRequest>,
) -> Result<StatusCode, StatusCode> {
    match container_service.bulk_delete_containers(&request.ids).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(AppError::BadRequest(msg)) => {
            tracing::warn!("Bad request in bulk_delete_containers: {}", msg);
            Err(StatusCode::BAD_REQUEST)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateContainersDisposedStatusRequest {
    pub ids: Vec<String>,
    pub is_disposed: bool,
}

pub async fn bulk_update_containers_disposed_status(
    State((_storage, _cable, _item_service, _loan, container_service)): State<crate::AppState>,
    Json(request): Json<BulkUpdateContainersDisposedStatusRequest>,
) -> Result<StatusCode, StatusCode> {
    match container_service
        .bulk_update_disposed_status(&request.ids, request.is_disposed)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
