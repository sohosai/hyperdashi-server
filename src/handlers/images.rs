use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::AppResult;
use crate::services::{ItemService, LoanService, StorageService, CableColorService};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadResponse {
    pub url: String,
    pub filename: String,
    pub size: usize,
}

pub async fn upload_image(
    State((_cable_color_service, _item_service, _loan_service, storage_service)): State<(Arc<CableColorService>, Arc<ItemService>, Arc<LoanService>, Arc<StorageService>)>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<ImageUploadResponse>)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "image" {
            let filename = field.file_name()
                .ok_or_else(|| crate::error::AppError::BadRequest("No filename provided".to_string()))?
                .to_string();
            
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            
            // 画像ファイルの検証
            if !is_image_content_type(&content_type) {
                return Err(crate::error::AppError::BadRequest(
                    "Only image files are allowed (JPEG, PNG, GIF, WebP)".to_string()
                ));
            }
            
            let data = field.bytes().await.map_err(|e| {
                crate::error::AppError::BadRequest(format!("Failed to read file data: {}", e))
            })?;
            
            // ファイルサイズ制限 (5MB)
            const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
            if data.len() > MAX_FILE_SIZE {
                return Err(crate::error::AppError::BadRequest(
                    "File size exceeds 5MB limit".to_string()
                ));
            }
            
            // ユニークなファイル名を生成
            let unique_filename = generate_unique_filename(&filename);
            
            // ストレージにアップロード
            let url = storage_service.upload(data.to_vec(), &unique_filename, &content_type).await?;
            
            return Ok((StatusCode::CREATED, Json(ImageUploadResponse {
                url,
                filename: unique_filename,
                size: data.len(),
            })));
        }
    }
    
    Err(crate::error::AppError::BadRequest("No image field found in multipart data".to_string()))
}

fn is_image_content_type(content_type: &str) -> bool {
    matches!(content_type, 
        "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/webp"
    )
}

fn generate_unique_filename(original_filename: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let extension = std::path::Path::new(original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg");
    
    format!("{}_{}.{}", timestamp, uuid::Uuid::new_v4(), extension)
}