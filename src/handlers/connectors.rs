use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{
    Connector, ConnectorsListResponse, CreateConnectorRequest, UpdateConnectorRequest,
};

#[derive(Deserialize)]
pub struct ConnectorsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    100 // Return more connectors by default since it's a master list
}

pub async fn list_connectors(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        connector_service,
        _tag_service,
    )): State<crate::AppState>,
    Query(params): Query<ConnectorsQuery>,
) -> AppResult<Json<ConnectorsListResponse>> {
    let response = connector_service
        .list_connectors(params.page, params.per_page)
        .await?;

    Ok(Json(response))
}

pub async fn get_connector(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        connector_service,
        _tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Connector>> {
    let connector = connector_service.get_connector(id).await?;
    Ok(Json(connector))
}

pub async fn create_connector(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        connector_service,
        _tag_service,
    )): State<crate::AppState>,
    Json(req): Json<CreateConnectorRequest>,
) -> AppResult<(StatusCode, Json<Connector>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let connector = connector_service.create_connector(req).await?;
    Ok((StatusCode::CREATED, Json(connector)))
}

pub async fn update_connector(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        connector_service,
        _tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateConnectorRequest>,
) -> AppResult<Json<Connector>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let connector = connector_service.update_connector(id, req).await?;
    Ok(Json(connector))
}

pub async fn delete_connector(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        connector_service,
        _tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    connector_service.delete_connector(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
