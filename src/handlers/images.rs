use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use axum::extract::Path;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadResponse {
    pub url: String,
    pub filename: String,
    pub size: usize,
}

pub async fn upload_image(
    State((storage_service, _cable_color_service, _item_service, _loan_service, _container_service)): State<crate::AppState>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<ImageUploadResponse>)> {
    tracing::info!("Starting image upload process");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to read multipart field: {}", e);
        crate::error::AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        tracing::debug!("Processing multipart field: '{}'", name);

        if name == "image" {
            let filename = field
                .file_name()
                .unwrap_or("image.jpg") // デフォルトファイル名を提供
                .to_string();

            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            tracing::info!(
                "Received file: filename='{}', content_type='{}'",
                filename,
                content_type
            );

            let content_type_valid = is_image_content_type(&content_type);
            let extension_valid = is_image_extension(&filename);

            tracing::info!(
                "Validation: content_type_valid={}, extension_valid={}",
                content_type_valid,
                extension_valid
            );

            // 画像ファイルの検証（Content-Typeまたは拡張子で判定）
            if !content_type_valid && !extension_valid {
                tracing::error!(
                    "File rejected: content-type='{}', filename='{}'",
                    content_type,
                    filename
                );
                return Err(crate::error::AppError::BadRequest(
                    format!("Only image files are allowed (JPEG, PNG, GIF, WebP). Got content-type: {}, filename: {}", content_type, filename)
                ));
            }

            // チャンクごとにデータを読み込み
            let mut data = Vec::new();
            let mut field = field;

            while let Some(chunk) = field.chunk().await.map_err(|e| {
                tracing::error!("Failed to read file chunk: {:?}", e);
                crate::error::AppError::BadRequest(format!("Failed to read file chunk: {}", e))
            })? {
                data.extend_from_slice(&chunk);

                // メモリ使用量制限のため、チャンクごとにサイズチェック
                let max_file_size = storage_service.get_max_file_size_bytes();
                if data.len() > max_file_size {
                    tracing::error!(
                        "File size too large during chunk reading: {} > {}",
                        data.len(),
                        max_file_size
                    );
                    return Err(crate::error::AppError::BadRequest(format!(
                        "File size exceeds {}MB limit",
                        max_file_size / (1024 * 1024)
                    )));
                }
            }

            tracing::info!("File data read successfully, size: {} bytes", data.len());

            // ユニークなファイル名を生成
            let unique_filename = generate_unique_filename(&filename);
            tracing::info!("Generated unique filename: {}", unique_filename);

            // ストレージにアップロード
            tracing::info!("Starting storage upload...");
            let url = storage_service
                .upload(data.to_vec(), &unique_filename, &content_type)
                .await
                .map_err(|e| {
                    tracing::error!("Storage upload failed: {}", e);
                    e
                })?;

            tracing::info!("Upload successful! URL: {}", url);

            return Ok((
                StatusCode::CREATED,
                Json(ImageUploadResponse {
                    url,
                    filename: unique_filename,
                    size: data.len(),
                }),
            ));
        }
    }

    tracing::error!("No image field found in multipart data");
    Err(crate::error::AppError::BadRequest(
        "No image field found in multipart data".to_string(),
    ))
}

pub async fn delete_image(
   State((storage_service, _, _, _, _)): State<crate::AppState>,
   Path(filename): Path<String>,
) -> AppResult<StatusCode> {
   tracing::info!("Attempting to delete image: {}", filename);

   storage_service.delete(&filename).await?;

   tracing::info!("Successfully deleted image: {}", filename);
   Ok(StatusCode::NO_CONTENT)
}

fn is_image_content_type(content_type: &str) -> bool {
    let is_valid = matches!(
        content_type,
        "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/webp" |
        "image/pjpeg" | // IE用JPEG
        "application/octet-stream" // ブラウザによってはこれで送信される
    );
    tracing::debug!("Content-Type '{}' validation: {}", content_type, is_valid);
    is_valid
}

fn is_image_extension(filename: &str) -> bool {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    let is_valid = matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp");
    tracing::debug!("File extension '{}' validation: {}", extension, is_valid);
    is_valid
}

fn generate_unique_filename(original_filename: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let extension = std::path::Path::new(original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg");

    format!("{}_{}.{}", timestamp, uuid::Uuid::new_v4(), extension)
}
