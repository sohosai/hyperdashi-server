use crate::error::AppError;
use crate::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GenerateLabelsRequest {
    pub quantity: u32,
    pub record_type: String, // "qr", "barcode", or "nothing"
}

#[derive(Debug, Serialize)]
pub struct GenerateLabelsResponse {
    pub visible_ids: Vec<String>,
}

pub async fn generate_labels(
    State(state): State<AppState>,
    Json(req): Json<GenerateLabelsRequest>,
) -> Result<Json<GenerateLabelsResponse>, AppError> {
    // Validate quantity
    if req.quantity == 0 || req.quantity > 1000 {
        return Err(AppError::BadRequest(
            "Quantity must be between 1 and 1000".to_string(),
        ));
    }

    // Validate record type
    let valid_types = ["qr", "barcode", "nothing"];
    if !valid_types.contains(&req.record_type.as_str()) {
        return Err(AppError::BadRequest("Invalid record type".to_string()));
    }

    // Generate sequential label IDs
    let visible_ids = state
        .2
        .generate_label_ids(req.quantity)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

    Ok(Json(GenerateLabelsResponse { visible_ids }))
}

#[derive(Debug, Serialize)]
pub struct LabelInfo {
    pub id: String,
    pub used: bool,
    pub item_name: Option<String>,
}

pub async fn get_label_info(
    State(state): State<AppState>,
) -> Result<Json<Vec<LabelInfo>>, AppError> {
    let labels = state
        .2
        .get_all_labels()
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

    Ok(Json(labels))
}
