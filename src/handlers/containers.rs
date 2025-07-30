use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{
    Container, ContainerWithItemCount, CreateContainerRequest, UpdateContainerRequest,
};
use crate::services::ContainerService;
use crate::ContainerAppState;

#[derive(Debug, Deserialize)]
pub struct ListContainersQuery {
    pub location: Option<String>,
    pub include_disposed: Option<bool>,
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

pub async fn container_create(
    State(state): State<ContainerAppState>,
    Json(request): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<CreateContainerResponse>), StatusCode> {
    if request.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let container_service = ContainerService::new(state.db.clone());

    match container_service.create_container(request).await {
        Ok(container) => Ok((
            StatusCode::CREATED,
            Json(CreateContainerResponse { container }),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn container_get(
    State(state): State<ContainerAppState>,
    Path(id): Path<String>,
) -> Result<Json<GetContainerResponse>, StatusCode> {
    let container_service = ContainerService::new(state.db.clone());

    match container_service.get_container(&id).await {
        Ok(container) => Ok(Json(GetContainerResponse { container })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn container_list(
    State(state): State<ContainerAppState>,
    Query(query): Query<ListContainersQuery>,
) -> Result<Json<ListContainersResponse>, StatusCode> {
    let container_service = ContainerService::new(state.db.clone());

    let location_filter = query.location.as_deref();
    let include_disposed = query.include_disposed.unwrap_or(false);

    match container_service
        .list_containers(location_filter, include_disposed)
        .await
    {
        Ok(containers) => Ok(Json(ListContainersResponse { containers })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn container_update(
    State(state): State<ContainerAppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateContainerRequest>,
) -> Result<Json<UpdateContainerResponse>, StatusCode> {
    if request.validate().is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let container_service = ContainerService::new(state.db.clone());

    match container_service.update_container(&id, request).await {
        Ok(container) => Ok(Json(UpdateContainerResponse { container })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn container_delete(
    State(state): State<ContainerAppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let container_service = ContainerService::new(state.db.clone());

    match container_service.delete_container(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn containers_by_location(
    State(state): State<ContainerAppState>,
    Path(location): Path<String>,
) -> Result<Json<ListContainersResponse>, StatusCode> {
    let container_service = ContainerService::new(state.db.clone());

    match container_service
        .get_containers_by_location(&location)
        .await
    {
        Ok(containers) => {
            let containers_with_count = containers
                .into_iter()
                .map(|container| {
                    ContainerWithItemCount {
                        container,
                        item_count: 0, // For this endpoint, we don't calculate item count
                    }
                })
                .collect();
            Ok(Json(ListContainersResponse {
                containers: containers_with_count,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
