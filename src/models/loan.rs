use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: i64,
    pub item_id: Uuid,
    pub student_number: String,
    pub student_name: String,
    pub organization: Option<String>,
    pub loan_date: DateTime<Utc>,
    pub return_date: Option<DateTime<Utc>>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateLoanRequest {
    pub item_id: Uuid,

    #[validate(length(min = 1, max = 20))]
    pub student_number: String,

    #[validate(length(min = 1, max = 100))]
    pub student_name: String,

    #[validate(length(max = 255))]
    pub organization: Option<String>,

    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ReturnLoanRequest {
    pub return_date: Option<DateTime<Utc>>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanWithItem {
    pub id: i64,
    pub item_id: Uuid,
    pub item_name: String,
    pub item_label_id: String,
    pub student_number: String,
    pub student_name: String,
    pub organization: Option<String>,
    pub loan_date: DateTime<Utc>,
    pub return_date: Option<DateTime<Utc>>,
    pub remarks: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoansListResponse {
    pub loans: Vec<LoanWithItem>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanFilters {
    pub item_id: Option<Uuid>,
    pub student_number: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
