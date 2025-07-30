use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateLoanRequest, Loan, LoanWithItem, LoansListResponse, ReturnLoanRequest};
use chrono::Utc;
use sqlx::Row;

pub struct LoanService {
    db: DatabasePool,
}

impl LoanService {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_loan(&self, req: CreateLoanRequest) -> AppResult<Loan> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // まず、物品が存在し、貸出可能かチェック
                let item_row = sqlx::query(
                    "SELECT id, name, is_on_loan, is_disposed FROM items WHERE id = $1",
                )
                .bind(req.item_id)
                .fetch_optional(pool)
                .await?;

                let item_row = item_row.ok_or_else(|| {
                    AppError::NotFound(format!("Item with id {} not found", req.item_id))
                })?;

                let is_on_loan: Option<bool> = item_row.try_get("is_on_loan").unwrap_or(None);
                let is_disposed: Option<bool> = item_row.try_get("is_disposed").unwrap_or(None);

                if is_on_loan.unwrap_or(false) {
                    return Err(AppError::BadRequest("Item is already on loan".to_string()));
                }

                if is_disposed.unwrap_or(false) {
                    return Err(AppError::BadRequest(
                        "Item is disposed and cannot be loaned".to_string(),
                    ));
                }

                // 貸出記録を作成
                let result = sqlx::query(
                    r#"
                    INSERT INTO loans (
                        item_id, student_number, student_name, organization, remarks
                    ) VALUES ($1, $2, $3, $4, $5)
                    RETURNING id
                    "#,
                )
                .bind(req.item_id)
                .bind(&req.student_number)
                .bind(&req.student_name)
                .bind(&req.organization)
                .bind(&req.remarks)
                .fetch_one(pool)
                .await?;

                let loan_id: i64 = result.get("id");

                // 物品の貸出状態を更新
                let now = Utc::now();
                sqlx::query("UPDATE items SET is_on_loan = true, updated_at = $2 WHERE id = $1")
                    .bind(req.item_id)
                    .bind(now)
                    .execute(pool)
                    .await?;

                self.get_loan(loan_id).await
            }
            DatabasePool::Sqlite(pool) => {
                // まず、物品が存在し、貸出可能かチェック
                let item_row = sqlx::query(
                    "SELECT id, name, is_on_loan, is_disposed FROM items WHERE id = ?1",
                )
                .bind(req.item_id)
                .fetch_optional(pool)
                .await?;

                let item_row = item_row.ok_or_else(|| {
                    AppError::NotFound(format!("Item with id {} not found", req.item_id))
                })?;

                let is_on_loan: Option<bool> = item_row.try_get("is_on_loan").unwrap_or(None);
                let is_disposed: Option<bool> = item_row.try_get("is_disposed").unwrap_or(None);

                if is_on_loan.unwrap_or(false) {
                    return Err(AppError::BadRequest("Item is already on loan".to_string()));
                }

                if is_disposed.unwrap_or(false) {
                    return Err(AppError::BadRequest(
                        "Item is disposed and cannot be loaned".to_string(),
                    ));
                }

                // 貸出記録を作成
                let result = sqlx::query!(
                    r#"
                    INSERT INTO loans (
                        item_id, student_number, student_name, organization, remarks
                    ) VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    req.item_id,
                    req.student_number,
                    req.student_name,
                    req.organization,
                    req.remarks
                )
                .execute(pool)
                .await?;

                // 物品の貸出状態を更新
                let now = Utc::now();
                sqlx::query!(
                    "UPDATE items SET is_on_loan = 1, updated_at = ?2 WHERE id = ?1",
                    req.item_id,
                    now
                )
                .execute(pool)
                .await?;

                let loan_id = result.last_insert_rowid();
                self.get_loan(loan_id).await
            }
        }
    }

    pub async fn get_loan(&self, id: i64) -> AppResult<Loan> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT
                        id, item_id, student_number, student_name, organization,
                        loan_date, return_date, remarks, created_at, updated_at
                    FROM loans
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Loan with id {} not found", id)))?;

                Ok(self.row_to_loan_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT
                        id, item_id, student_number, student_name, organization,
                        loan_date, return_date, remarks, created_at, updated_at
                    FROM loans
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Loan with id {} not found", id)))?;

                Ok(self.row_to_loan(row))
            }
        }
    }

    pub async fn list_loans(
        &self,
        page: u32,
        per_page: u32,
        item_id: Option<i64>,
        student_number: Option<String>,
        active_only: Option<bool>,
    ) -> AppResult<LoansListResponse> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Build dynamic WHERE query for PostgreSQL
                let mut where_conditions = Vec::new();
                let mut param_index = 1;

                if item_id.is_some() {
                    where_conditions.push(format!("l.item_id = ${}", param_index));
                    param_index += 1;
                }

                if student_number.is_some() {
                    where_conditions.push(format!("l.student_number = ${}", param_index));
                    param_index += 1;
                }

                if let Some(active) = active_only {
                    if active {
                        where_conditions.push("l.return_date IS NULL".to_string());
                    } else {
                        where_conditions.push("l.return_date IS NOT NULL".to_string());
                    }
                }

                let where_clause = if where_conditions.is_empty() {
                    String::new()
                } else {
                    format!("WHERE {}", where_conditions.join(" AND "))
                };

                let query_str = format!(
                    r#"
                    SELECT
                        l.id, l.item_id, l.student_number, l.student_name, l.organization,
                        l.loan_date, l.return_date, l.remarks, l.created_at, l.updated_at,
                        i.name as item_name, i.label_id as item_label_id
                    FROM loans l
                    INNER JOIN items i ON l.item_id = i.id
                    {}
                    ORDER BY l.created_at DESC
                    LIMIT ${} OFFSET ${}
                    "#,
                    where_clause,
                    param_index,
                    param_index + 1
                );

                let count_query_str = format!(
                    "SELECT COUNT(*) as count FROM loans l INNER JOIN items i ON l.item_id = i.id {}",
                    where_clause
                );

                let mut query = sqlx::query(&query_str);
                let mut count_query = sqlx::query(&count_query_str);

                if let Some(item_id_val) = item_id {
                    query = query.bind(item_id_val);
                    count_query = count_query.bind(item_id_val);
                }

                if let Some(student_number_val) = &student_number {
                    query = query.bind(student_number_val);
                    count_query = count_query.bind(student_number_val);
                }

                query = query.bind(limit).bind(offset);

                let rows = query.fetch_all(pool).await?;
                let loans: Vec<LoanWithItem> = rows
                    .into_iter()
                    .map(|row| self.row_to_loan_with_item_postgres(row))
                    .collect();

                let count_row = count_query.fetch_one(pool).await?;
                let total: i64 = count_row.get("count");

                Ok(LoansListResponse {
                    loans,
                    total,
                    page,
                    per_page,
                })
            }
            DatabasePool::Sqlite(pool) => {
                // フィルタリング機能を実装
                let (loans, total) = if item_id.is_none()
                    && student_number.is_none()
                    && active_only.is_none()
                {
                    // フィルターなし
                    let rows = sqlx::query(
                        r#"
                        SELECT
                            l.id, l.item_id, l.student_number, l.student_name, l.organization,
                            l.loan_date, l.return_date, l.remarks, l.created_at, l.updated_at,
                            i.name as item_name, i.label_id as item_label_id
                        FROM loans l
                        INNER JOIN items i ON l.item_id = i.id
                        ORDER BY l.created_at DESC
                        LIMIT ?1 OFFSET ?2
                        "#,
                    )
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?;

                    let loans: Vec<LoanWithItem> = rows
                        .into_iter()
                        .map(|row| self.row_to_loan_with_item(row))
                        .collect();

                    let count_row = sqlx::query("SELECT COUNT(*) as count FROM loans")
                        .fetch_one(pool)
                        .await?;
                    let total: i64 = count_row.get("count");

                    (loans, total)
                } else {
                    // フィルターあり - 動的クエリを構築
                    let mut where_conditions = Vec::new();

                    // 物品IDフィルター
                    if item_id.is_some() {
                        where_conditions.push("l.item_id = ?".to_string());
                    }

                    // 学籍番号フィルター
                    if student_number.is_some() {
                        where_conditions.push("l.student_number = ?".to_string());
                    }

                    // アクティブ貸出のみフィルター
                    if let Some(true) = active_only {
                        where_conditions.push("l.return_date IS NULL".to_string());
                    } else if let Some(false) = active_only {
                        where_conditions.push("l.return_date IS NOT NULL".to_string());
                    }

                    let where_clause = if where_conditions.is_empty() {
                        String::new()
                    } else {
                        format!("WHERE {}", where_conditions.join(" AND "))
                    };

                    let query_str = format!(
                        r#"
                        SELECT
                            l.id, l.item_id, l.student_number, l.student_name, l.organization,
                            l.loan_date, l.return_date, l.remarks, l.created_at, l.updated_at,
                            i.name as item_name, i.label_id as item_label_id
                        FROM loans l
                        INNER JOIN items i ON l.item_id = i.id
                        {}
                        ORDER BY l.created_at DESC
                        LIMIT ? OFFSET ?
                        "#,
                        where_clause
                    );

                    let count_query_str = format!(
                        "SELECT COUNT(*) as count FROM loans l INNER JOIN items i ON l.item_id = i.id {}",
                        where_clause
                    );

                    // パラメーターをバインド
                    let mut query = sqlx::query(&query_str);
                    let mut count_query = sqlx::query(&count_query_str);

                    // 物品IDフィルター
                    if let Some(id) = item_id {
                        query = query.bind(id);
                        count_query = count_query.bind(id);
                    }

                    // 学籍番号フィルター
                    if let Some(ref number) = student_number {
                        query = query.bind(number);
                        count_query = count_query.bind(number);
                    }

                    // LIMIT/OFFSETをバインド（active_onlyは既にWHERE句に含まれている）
                    query = query.bind(limit).bind(offset);

                    let rows = query.fetch_all(pool).await?;
                    let loans: Vec<LoanWithItem> = rows
                        .into_iter()
                        .map(|row| self.row_to_loan_with_item(row))
                        .collect();

                    let count_row = count_query.fetch_one(pool).await?;
                    let total: i64 = count_row.get("count");

                    (loans, total)
                };

                Ok(LoansListResponse {
                    loans,
                    total,
                    page,
                    per_page,
                })
            }
        }
    }

    pub async fn return_loan(&self, id: i64, req: ReturnLoanRequest) -> AppResult<Loan> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // 貸出記録が存在し、未返却かチェック
                let loan_row =
                    sqlx::query("SELECT id, item_id, return_date FROM loans WHERE id = $1")
                        .bind(id)
                        .fetch_optional(pool)
                        .await?;

                let loan_row = loan_row
                    .ok_or_else(|| AppError::NotFound(format!("Loan with id {} not found", id)))?;

                let return_date_check: Option<chrono::DateTime<chrono::Utc>> =
                    loan_row.try_get("return_date").unwrap_or(None);
                if return_date_check.is_some() {
                    return Err(AppError::BadRequest(
                        "Loan has already been returned".to_string(),
                    ));
                }

                let item_id: i64 = loan_row.get("item_id");
                let return_date = req.return_date.unwrap_or_else(chrono::Utc::now);
                let now = chrono::Utc::now();

                // 貸出記録を更新
                sqlx::query(
                    "UPDATE loans SET return_date = $1, remarks = $2, updated_at = $3 WHERE id = $4"
                )
                .bind(return_date)
                .bind(&req.remarks)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?;

                // 物品の貸出状態を更新
                sqlx::query("UPDATE items SET is_on_loan = false, updated_at = $1 WHERE id = $2")
                    .bind(now)
                    .bind(item_id)
                    .execute(pool)
                    .await?;

                // 更新された貸出記録を取得
                self.get_loan(id).await
            }
            DatabasePool::Sqlite(pool) => {
                // 貸出記録が存在し、未返却かチェック
                let loan_row =
                    sqlx::query("SELECT id, item_id, return_date FROM loans WHERE id = ?1")
                        .bind(id)
                        .fetch_optional(pool)
                        .await?;

                let loan_row = loan_row
                    .ok_or_else(|| AppError::NotFound(format!("Loan with id {} not found", id)))?;

                let return_date_check: Option<chrono::NaiveDateTime> =
                    loan_row.try_get("return_date").unwrap_or(None);
                if return_date_check.is_some() {
                    return Err(AppError::BadRequest(
                        "Loan has already been returned".to_string(),
                    ));
                }

                let return_date = req.return_date.unwrap_or_else(Utc::now);
                let now = Utc::now();

                // 貸出記録を更新
                sqlx::query!(
                    "UPDATE loans SET return_date = ?2, remarks = ?3, updated_at = ?4 WHERE id = ?1",
                    id,
                    return_date,
                    req.remarks,
                    now
                )
                .execute(pool)
                .await?;

                let item_id: i64 = loan_row.try_get("item_id").map_err(|_| {
                    AppError::InternalServerError("Loan record missing item_id".to_string())
                })?;

                // 物品の貸出状態を更新
                sqlx::query!(
                    "UPDATE items SET is_on_loan = 0, updated_at = ?2 WHERE id = ?1",
                    item_id,
                    now
                )
                .execute(pool)
                .await?;

                self.get_loan(id).await
            }
        }
    }

    fn row_to_loan(&self, row: sqlx::sqlite::SqliteRow) -> Loan {
        Loan {
            id: row.get("id"),
            item_id: row.get("item_id"),
            student_number: row.get("student_number"),
            student_name: row.get("student_name"),
            organization: row.get("organization"),
            loan_date: row.get("loan_date"),
            return_date: row.get("return_date"),
            remarks: row.get("remarks"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_loan_with_item(&self, row: sqlx::sqlite::SqliteRow) -> LoanWithItem {
        LoanWithItem {
            id: row.get("id"),
            item_id: row.get("item_id"),
            item_name: row.get("item_name"),
            item_label_id: row.get("item_label_id"),
            student_number: row.get("student_number"),
            student_name: row.get("student_name"),
            organization: row.get("organization"),
            loan_date: row.get("loan_date"),
            return_date: row.get("return_date"),
            remarks: row.get("remarks"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_loan_postgres(&self, row: sqlx::postgres::PgRow) -> Loan {
        Loan {
            id: row.get("id"),
            item_id: row.get("item_id"),
            student_number: row.get("student_number"),
            student_name: row.get("student_name"),
            organization: row.get("organization"),
            loan_date: row.get("loan_date"),
            return_date: row.get("return_date"),
            remarks: row.get("remarks"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_loan_with_item_postgres(&self, row: sqlx::postgres::PgRow) -> LoanWithItem {
        LoanWithItem {
            id: row.get("id"),
            item_id: row.get("item_id"),
            item_name: row.get("item_name"),
            item_label_id: row.get("item_label_id"),
            student_number: row.get("student_number"),
            student_name: row.get("student_name"),
            organization: row.get("organization"),
            loan_date: row.get("loan_date"),
            return_date: row.get("return_date"),
            remarks: row.get("remarks"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
