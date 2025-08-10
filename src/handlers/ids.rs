use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize)]
pub struct IdCheckResponse {
    pub exists: bool,
    pub found_in: Vec<String>,
    pub duplicates: Vec<DuplicateItem>,
}

#[derive(Serialize)]
pub struct DuplicateItem {
    pub name: String,
    pub item_type: String,
}

pub async fn check_global_id(
    Path(id): Path<String>,
    State((_storage_service, _cable_color_service, item_service, _loan_service, container_service)): State<crate::AppState>,
) -> AppResult<Json<IdCheckResponse>> {
    let mut found_in = Vec::new();
    let mut duplicates = Vec::new();
    let mut exists = false;

    // Check in items
    if let Ok(item) = item_service.get_item_by_label(&id).await {
        found_in.push("items".to_string());
        duplicates.push(DuplicateItem {
            name: item.name,
            item_type: "item".to_string(),
        });
        exists = true;
    }

    // Check in containers
    if let Ok(container) = container_service.get_container(&id).await {
        found_in.push("containers".to_string());
        duplicates.push(DuplicateItem {
            name: container.name,
            item_type: "container".to_string(),
        });
        exists = true;
    }

    Ok(Json(IdCheckResponse {
        exists,
        found_in,
        duplicates,
    }))
}
