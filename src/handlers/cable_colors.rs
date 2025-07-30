use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{CableColor, CableColorsListResponse, CreateCableColorRequest, UpdateCableColorRequest};
use crate::services::{CableColorService, ItemService, LoanService, StorageService};

#[derive(Deserialize)]
pub struct CableColorsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

pub async fn list_cable_colors(
    State((_storage_service, cable_color_service, _item_service, _loan_service)): State<(Arc<StorageService>, Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>)>,
    Query(params): Query<CableColorsQuery>,
) -> AppResult<Json<CableColorsListResponse>> {
    let response = cable_color_service
        .list_cable_colors(params.page, params.per_page)
        .await?;

    Ok(Json(response))
}

pub async fn get_cable_color(
    State((_storage_service, cable_color_service, _item_service, _loan_service)): State<(Arc<StorageService>, Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>)>,
    Path(id): Path<i64>,
) -> AppResult<Json<CableColor>> {
    let cable_color = cable_color_service.get_cable_color(id).await?;
    Ok(Json(cable_color))
}

pub async fn create_cable_color(
    State((_storage_service, cable_color_service, _item_service, _loan_service)): State<(Arc<StorageService>, Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>)>,
    Json(req): Json<CreateCableColorRequest>,
) -> AppResult<(StatusCode, Json<CableColor>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let cable_color = cable_color_service.create_cable_color(req).await?;
    Ok((StatusCode::CREATED, Json(cable_color)))
}

pub async fn update_cable_color(
    State((_storage_service, cable_color_service, _item_service, _loan_service)): State<(Arc<StorageService>, Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>)>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCableColorRequest>,
) -> AppResult<Json<CableColor>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let cable_color = cable_color_service.update_cable_color(id, req).await?;
    Ok(Json(cable_color))
}

pub async fn delete_cable_color(
    State((_storage_service, cable_color_service, _item_service, _loan_service)): State<(Arc<StorageService>, Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>)>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    cable_color_service.delete_cable_color(id).await?;
    Ok(StatusCode::NO_CONTENT)
}