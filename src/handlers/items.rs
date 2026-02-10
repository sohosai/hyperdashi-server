use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{CreateItemRequest, Item, ItemsListResponse, UpdateItemRequest};

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
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
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

pub async fn export_items_csv(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Query(params): Query<ItemsQuery>,
) -> AppResult<(HeaderMap, String)> {
    let items = item_service
        .list_items_for_csv(
            params.search,
            params.is_on_loan,
            params.is_disposed,
            params.container_id,
            params.storage_type,
        )
        .await?;

    let csv = items_to_csv(&items);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"items.csv\""),
    );

    Ok((headers, csv))
}

pub async fn get_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = item_service.get_item(id).await?;
    Ok(Json(item))
}

pub async fn get_item_by_label(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Path(label_id): Path<String>,
) -> AppResult<Json<Item>> {
    let item = item_service.get_item_by_label(&label_id).await?;
    Ok(Json(item))
}

pub async fn create_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Json(req): Json<CreateItemRequest>,
) -> AppResult<(StatusCode, Json<Item>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let item = item_service.create_item(req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> AppResult<Json<Item>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let item = item_service.update_item(id, req).await?;
    Ok(Json(item))
}

pub async fn delete_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    item_service.delete_item(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn dispose_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Item>> {
    let item = item_service.dispose_item(id).await?;
    Ok(Json(item))
}

pub async fn undispose_item(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
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
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
) -> AppResult<Json<SuggestionsResponse>> {
    let suggestions = item_service.get_connection_names_suggestions().await?;
    Ok(Json(SuggestionsResponse { suggestions }))
}

pub async fn get_storage_locations_suggestions(
    State((_storage_service, _cable_color_service, item_service, _loan_service, _container_service, _connector_service, _tag_service)): State<crate::AppState>,
) -> AppResult<Json<SuggestionsResponse>> {
    let suggestions = item_service.get_storage_locations_suggestions().await?;
    Ok(Json(SuggestionsResponse { suggestions }))
}

use axum::extract::Multipart;

pub async fn add_item_image(
    State((storage, _cable, item_service, _loan, _container, _connector, _tag)): State<crate::AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Item>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "image" {
            let file_name = field.file_name().unwrap_or("image.jpg").to_string();
            let content_type = field.content_type().unwrap_or("image/jpeg").to_string();
            let data = field.bytes().await.unwrap();
            
            match storage.upload(data.to_vec(), &file_name, &content_type).await {
                Ok(image_url) => {
                    match item_service.update_item_image(&id, &image_url).await {
                        Ok(item) => return Ok(Json(item)),
                        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                    }
                }
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteItemsRequest {
    pub ids: Vec<String>,
}

pub async fn bulk_delete_items(
    State((_storage, _cable, item_service, _loan, _container, _connector, _tag)): State<crate::AppState>,
    Json(request): Json<BulkDeleteItemsRequest>,
) -> AppResult<StatusCode> {
    item_service.bulk_delete_items(&request.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateItemsDisposedStatusRequest {
    pub ids: Vec<String>,
    pub is_disposed: bool,
}

pub async fn bulk_update_items_disposed_status(
    State((_storage, _cable, item_service, _loan, _container, _connector, _tag)): State<crate::AppState>,
    Json(request): Json<BulkUpdateItemsDisposedStatusRequest>,
) -> AppResult<StatusCode> {
    item_service
        .bulk_update_disposed_status(&request.ids, request.is_disposed)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn csv_escape(value: &str) -> String {
    let needs_quotes =
        value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r');
    if !needs_quotes {
        return value.to_string();
    }

    let escaped = value.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}

fn opt_bool_to_string(value: Option<bool>) -> String {
    match value {
        Some(true) => "TRUE".to_string(),
        Some(false) => "FALSE".to_string(),
        None => "".to_string(),
    }
}

fn opt_vec_to_string(value: &Option<Vec<String>>) -> String {
    value.as_ref().map(|v| v.join(";")).unwrap_or_default()
}

fn items_to_csv(items: &[Item]) -> String {
    // NOTE: keep header labels stable because users may import by column name.
    let headers = [
        "ID",
        "名称",
        "ラベルID",
        "型番",
        "保管場所",
        "コンテナID",
        "保管方法",
        "貸出中",
        "廃棄済み",
        "購入年",
        "購入金額",
        "耐用年数",
        "減価償却対象",
        "接続端子",
        "ケーブル色",
        "QR/バーコード種別",
        "画像URL",
        "備考",
        "登録日時",
        "更新日時",
    ];

    let mut out = String::new();
    out.push_str(&headers.join(","));
    out.push_str("\r\n");

    for item in items {
        let fields = [
            item.id.to_string(),
            item.name.clone(),
            item.label_id.clone(),
            item.model_number.clone().unwrap_or_default(),
            item.storage_location.clone().unwrap_or_default(),
            item.container_id.clone().unwrap_or_default(),
            item.storage_type.clone(),
            opt_bool_to_string(item.is_on_loan),
            opt_bool_to_string(item.is_disposed),
            item.purchase_year.map(|v| v.to_string()).unwrap_or_default(),
            item.purchase_amount.map(|v| v.to_string()).unwrap_or_default(),
            item.durability_years.map(|v| v.to_string()).unwrap_or_default(),
            opt_bool_to_string(item.is_depreciation_target),
            opt_vec_to_string(&item.connection_names),
            opt_vec_to_string(&item.cable_color_pattern),
            item.qr_code_type.clone().unwrap_or_default(),
            item.image_url.clone().unwrap_or_default(),
            item.remarks.clone().unwrap_or_default(),
            item.created_at.to_rfc3339(),
            item.updated_at.to_rfc3339(),
        ];

        let escaped_row: Vec<String> = fields.iter().map(|v| csv_escape(v)).collect();
        out.push_str(&escaped_row.join(","));
        out.push_str("\r\n");
    }

    out
}
