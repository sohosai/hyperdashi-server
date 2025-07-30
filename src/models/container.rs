use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub location: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_disposed: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateContainerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateContainerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub location: Option<String>,
    pub is_disposed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerWithItemCount {
    #[serde(flatten)]
    pub container: Container,
    pub item_count: i64,
}