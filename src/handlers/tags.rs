use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{CreateTagRequest, ItemTagsRequest, Tag, TagsListResponse, UpdateTagRequest};

#[derive(Deserialize)]
pub struct TagsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    100
}

pub async fn list_tags(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Query(params): Query<TagsQuery>,
) -> AppResult<Json<TagsListResponse>> {
    let response = tag_service.list_tags(params.page, params.per_page).await?;

    Ok(Json(response))
}

pub async fn get_tag(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Tag>> {
    let tag = tag_service.get_tag(id).await?;
    Ok(Json(tag))
}

pub async fn create_tag(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Json(req): Json<CreateTagRequest>,
) -> AppResult<(StatusCode, Json<Tag>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let tag = tag_service.create_tag(req).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn update_tag(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTagRequest>,
) -> AppResult<Json<Tag>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let tag = tag_service.update_tag(id, req).await?;
    Ok(Json(tag))
}

pub async fn delete_tag(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    tag_service.delete_tag(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Item-tag association endpoints
pub async fn get_item_tags(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Path(item_id): Path<String>,
) -> AppResult<Json<Vec<Tag>>> {
    let tags = tag_service.get_item_tags(&item_id).await?;
    Ok(Json(tags))
}

pub async fn set_item_tags(
    State((
        _storage_service,
        _cable_color_service,
        _item_service,
        _loan_service,
        _container_service,
        _connector_service,
        tag_service,
    )): State<crate::AppState>,
    Path(item_id): Path<String>,
    Json(req): Json<ItemTagsRequest>,
) -> AppResult<Json<Vec<Tag>>> {
    let tags = tag_service.set_item_tags(&item_id, req.tag_ids).await?;
    Ok(Json(tags))
}
