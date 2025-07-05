use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct CableColor {
    pub id: i64,
    pub name: String,
    pub hex_code: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCableColorRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(regex(path = "*crate::models::cable_color::HEX_COLOR_REGEX"))]
    pub hex_code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCableColorRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(regex(path = "*crate::models::cable_color::HEX_COLOR_REGEX"))]
    pub hex_code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CableColorsListResponse {
    pub cable_colors: Vec<CableColor>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref HEX_COLOR_REGEX: Regex = Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();
}