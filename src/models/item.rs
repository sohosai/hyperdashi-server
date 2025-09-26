use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub label_id: String,
    pub model_number: Option<String>,
    pub remarks: Option<String>,
    pub purchase_year: Option<i32>,
    pub purchase_amount: Option<f32>,
    pub durability_years: Option<i32>,
    pub is_depreciation_target: Option<bool>,
    pub connection_names: Option<Vec<String>>,
    pub cable_color_pattern: Option<Vec<String>>,
    pub storage_location: Option<String>,
    pub container_id: Option<String>,
    pub storage_type: String, // "location" or "container"
    pub is_on_loan: Option<bool>,
    pub qr_code_type: Option<String>,
    pub is_disposed: Option<bool>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateItemRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 1, max = 50))]
    pub label_id: String,

    #[validate(length(max = 255))]
    pub model_number: Option<String>,

    pub remarks: Option<String>,

    #[validate(range(min = 1900, max = 2100))]
    pub purchase_year: Option<i32>,

    pub purchase_amount: Option<f32>,

    #[validate(range(min = 1, max = 100))]
    pub durability_years: Option<i32>,

    pub is_depreciation_target: Option<bool>,

    pub connection_names: Option<Vec<String>>,

    pub cable_color_pattern: Option<Vec<String>>,

    pub storage_location: Option<String>,

    pub container_id: Option<String>,

    pub storage_type: Option<String>, // "location" or "container"

    pub qr_code_type: Option<String>,

    #[validate(url)]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateItemRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    #[validate(length(min = 1, max = 50))]
    pub label_id: Option<String>,

    #[validate(length(max = 255))]
    pub model_number: Option<String>,

    pub remarks: Option<String>,

    #[validate(range(min = 1900, max = 2100))]
    pub purchase_year: Option<i32>,

    pub purchase_amount: Option<f32>,

    #[validate(range(min = 1, max = 100))]
    pub durability_years: Option<i32>,

    pub is_depreciation_target: Option<bool>,

    pub connection_names: Option<Vec<String>>,

    pub cable_color_pattern: Option<Vec<String>>,

    pub storage_location: Option<String>,

    pub container_id: Option<String>,

    pub storage_type: Option<String>, // "location" or "container"

    pub is_on_loan: Option<bool>,

    pub qr_code_type: Option<String>,

    pub is_disposed: Option<bool>,

    #[validate(url)]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsListResponse {
    pub items: Vec<Item>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
