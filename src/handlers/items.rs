use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{CreateItemRequest, Item, ItemsListResponse, UpdateItemRequest};
use crate::services::{CableColorService, ItemService, LoanService, StorageService};

#[derive(Deserialize)]
pub struct ItemsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub search: Option<String>,
    pub is_on_loan: Option<bool>,
    pub is_disposed: Option<bool>,
    pub container_id: Option<String>,
    pub storage_type: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

pub async fn list_items(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Query(params): Query<ItemsQuery>,
) -> AppResult<Json<ItemsListResponse>> {
    let response = item_service
        .list_items(
            params.page,
            params.per_page,
            params.search,
            params.is_on_loan,
            params.is_disposed,
            params.container_id,
            params.storage_type,
        )
        .await?;

    Ok(Json(response))
}

pub async fn get_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = item_service.get_item(id).await?;
    Ok(Json(item))
}

pub async fn get_item_by_label(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(label_id): Path<String>,
) -> AppResult<Json<Item>> {
    let item = item_service.get_item_by_label(&label_id).await?;
    Ok(Json(item))
}

pub async fn create_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Json(req): Json<CreateItemRequest>,
) -> AppResult<(StatusCode, Json<Item>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let item = item_service.create_item(req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> AppResult<Json<Item>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let item = item_service.update_item(id, req).await?;
    Ok(Json(item))
}

pub async fn delete_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    item_service.delete_item(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn dispose_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = item_service.dispose_item(id).await?;
    Ok(Json(item))
}

pub async fn undispose_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = item_service.undispose_item(id).await?;
    Ok(Json(item))
}

#[derive(Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<String>,
}

pub async fn get_connection_names_suggestions(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
) -> AppResult<Json<SuggestionsResponse>> {
    let suggestions = item_service.get_connection_names_suggestions().await?;
    Ok(Json(SuggestionsResponse { suggestions }))
}

pub async fn get_storage_locations_suggestions(
    State((_storage_service, _cable_color_service, item_service, _loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
) -> AppResult<Json<SuggestionsResponse>> {
    let suggestions = item_service.get_storage_locations_suggestions().await?;
    Ok(Json(SuggestionsResponse { suggestions }))
}
