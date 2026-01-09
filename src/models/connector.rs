use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: i64,
    pub name: String,
    pub gender: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateConnectorRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub gender: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateConnectorRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub gender: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectorsListResponse {
    pub connectors: Vec<Connector>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
