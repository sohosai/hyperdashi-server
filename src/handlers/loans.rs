use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::error::AppResult;
use crate::models::{CreateLoanRequest, Loan, LoansListResponse, ReturnLoanRequest};
use crate::services::{CableColorService, ItemService, LoanService, StorageService};

#[derive(Deserialize)]
pub struct LoansQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub item_id: Option<i64>,
    pub student_number: Option<String>,
    pub active_only: Option<bool>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

pub async fn list_loans(
    State((_storage_service, _cable_color_service, _item_service, loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Query(params): Query<LoansQuery>,
) -> AppResult<Json<LoansListResponse>> {
    let response = loan_service
        .list_loans(
            params.page,
            params.per_page,
            params.item_id,
            params.student_number,
            params.active_only,
        )
        .await?;

    Ok(Json(response))
}

pub async fn get_loan(
    State((_storage_service, _cable_color_service, _item_service, loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<i64>,
) -> AppResult<Json<Loan>> {
    let loan = loan_service.get_loan(id).await?;
    Ok(Json(loan))
}

pub async fn create_loan(
    State((_storage_service, _cable_color_service, _item_service, loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Json(req): Json<CreateLoanRequest>,
) -> AppResult<(StatusCode, Json<Loan>)> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let loan = loan_service.create_loan(req).await?;
    Ok((StatusCode::CREATED, Json(loan)))
}

pub async fn return_loan(
    State((_storage_service, _cable_color_service, _item_service, loan_service)): State<(
        Arc<StorageService>,
        Arc<CableColorService>,
        Arc<ItemService>,
        Arc<LoanService>,
    )>,
    Path(id): Path<i64>,
    Json(req): Json<ReturnLoanRequest>,
) -> AppResult<Json<Loan>> {
    req.validate()
        .map_err(|e| crate::error::AppError::ValidationError(e.to_string()))?;

    let loan = loan_service.return_loan(id, req).await?;
    Ok(Json(loan))
}
