use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateItemRequest, Item, ItemsListResponse, UpdateItemRequest};
use chrono::Utc;
use sqlx::Row;

pub struct ItemService {
    db: DatabasePool,
}

impl ItemService {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_item(&self, req: CreateItemRequest) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let connection_names = req.connection_names
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());
                let cable_color_pattern = req.cable_color_pattern
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());
                let storage_location = req.storage_location;
                let is_depreciation_target = req.is_depreciation_target.unwrap_or(false);

                let storage_type = req.storage_type.as_ref().unwrap_or(&"location".to_string()).clone();
                
                let result = sqlx::query(
                    r#"
                    INSERT INTO items (
                        name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target, connection_names,
                        cable_color_pattern, storage_location, container_id, storage_type, qr_code_type, image_url
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    RETURNING id
                    "#
                )
                .bind(&req.name)
                .bind(&req.label_id)
                .bind(&req.model_number)
                .bind(&req.remarks)
                .bind(&req.purchase_year)
                .bind(&req.purchase_amount)
                .bind(&req.durability_years)
                .bind(is_depreciation_target)
                .bind(&connection_names)
                .bind(&cable_color_pattern)
                .bind(&storage_location)
                .bind(&req.container_id)
                .bind(&storage_type)
                .bind(&req.qr_code_type)
                .bind(&req.image_url)
                .fetch_one(pool)
                .await?;

                let id: i64 = result.get("id");
                self.get_item(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let connection_names = req.connection_names
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());
                let cable_color_pattern = req.cable_color_pattern
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());
                let storage_location = req.storage_location;
                let is_depreciation_target = req.is_depreciation_target.unwrap_or(false);

                let storage_type = req.storage_type.as_ref().unwrap_or(&"location".to_string()).clone();
                
                let result = sqlx::query!(
                    r#"
                    INSERT INTO items (
                        name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target, connection_names,
                        cable_color_pattern, storage_location, container_id, storage_type, qr_code_type, image_url
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                    "#,
                    req.name,
                    req.label_id,
                    req.model_number,
                    req.remarks,
                    req.purchase_year,
                    req.purchase_amount,
                    req.durability_years,
                    is_depreciation_target,
                    connection_names,
                    cable_color_pattern,
                    storage_location,
                    req.container_id,
                    storage_type,
                    req.qr_code_type,
                    req.image_url
                )
                .execute(pool)
                .await?;

                let id = result.last_insert_rowid();
                self.get_item(id).await
            }
        }
    }

    pub async fn get_item(&self, id: i64) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT 
                        id, name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target,
                        connection_names, cable_color_pattern, storage_location,
                        container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                        created_at, updated_at
                    FROM items 
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Item with id {} not found", id)))?;

                Ok(self.row_to_item_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT 
                        id, name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target,
                        connection_names, cable_color_pattern, storage_location,
                        container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                        created_at, updated_at
                    FROM items 
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Item with id {} not found", id)))?;

                Ok(self.row_to_item(row))
            }
        }
    }

    pub async fn get_item_by_label(&self, label_id: &str) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT 
                        id, name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target,
                        connection_names, cable_color_pattern, storage_location,
                        container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                        created_at, updated_at
                    FROM items 
                    WHERE label_id = $1
                    "#,
                )
                .bind(label_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Item with label_id {} not found", label_id)))?;

                Ok(self.row_to_item_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT 
                        id, name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target,
                        connection_names, cable_color_pattern, storage_location,
                        container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                        created_at, updated_at
                    FROM items 
                    WHERE label_id = ?1
                    "#,
                )
                .bind(label_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Item with label_id {} not found", label_id)))?;

                Ok(self.row_to_item(row))
            }
        }
    }

    pub async fn list_items(
        &self,
        page: u32,
        per_page: u32,
        search: Option<String>,
        is_on_loan: Option<bool>,
        is_disposed: Option<bool>,
        container_id: Option<String>,
        storage_type: Option<String>,
    ) -> AppResult<ItemsListResponse> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        match &self.db {
            DatabasePool::Postgres(pool) => {
                // 動的WHEREクエリを構築（PostgreSQL版）
                let mut where_conditions = Vec::new();
                let mut param_index = 1;

                // 検索条件
                if search.is_some() {
                    where_conditions.push(format!("(name ILIKE ${} OR label_id ILIKE ${} OR model_number ILIKE ${} OR remarks ILIKE ${})", param_index, param_index+1, param_index+2, param_index+3));
                    param_index += 4;
                }

                // 貸出状態フィルター
                if is_on_loan.is_some() {
                    where_conditions.push(format!("is_on_loan = ${}", param_index));
                    param_index += 1;
                }

                // 廃棄状態フィルター
                if is_disposed.is_some() {
                    where_conditions.push(format!("is_disposed = ${}", param_index));
                    param_index += 1;
                }

                // コンテナIDフィルター
                if container_id.is_some() {
                    where_conditions.push(format!("container_id = ${}", param_index));
                    param_index += 1;
                }

                // 保管タイプフィルター
                if storage_type.is_some() {
                    where_conditions.push(format!("storage_type = ${}", param_index));
                    param_index += 1;
                }

                let where_clause = if where_conditions.is_empty() {
                    String::new()
                } else {
                    format!("WHERE {}", where_conditions.join(" AND "))
                };

                let query_str = format!(
                    r#"
                    SELECT 
                        id, name, label_id, model_number, remarks, purchase_year,
                        purchase_amount, durability_years, is_depreciation_target,
                        connection_names, cable_color_pattern, storage_location,
                        container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                        created_at, updated_at
                    FROM items 
                    {}
                    ORDER BY created_at DESC 
                    LIMIT ${} OFFSET ${}
                    "#,
                    where_clause, param_index, param_index+1
                );

                let count_query_str = format!("SELECT COUNT(*) as count FROM items {}", where_clause);

                // パラメーターをバインド
                let mut query = sqlx::query(&query_str);
                let mut count_query = sqlx::query(&count_query_str);

                // 検索条件
                if let Some(search_term) = &search {
                    let search_pattern = format!("%{}%", search_term);
                    query = query.bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone());
                    count_query = count_query.bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern);
                }

                // 貸出状態フィルター
                if let Some(loan_status) = is_on_loan {
                    query = query.bind(loan_status);
                    count_query = count_query.bind(loan_status);
                }

                // 廃棄状態フィルター
                if let Some(disposed_status) = is_disposed {
                    query = query.bind(disposed_status);
                    count_query = count_query.bind(disposed_status);
                }

                // コンテナIDフィルター
                if let Some(container_id_val) = &container_id {
                    query = query.bind(container_id_val);
                    count_query = count_query.bind(container_id_val);
                }

                // 保管タイプフィルター
                if let Some(storage_type_val) = &storage_type {
                    query = query.bind(storage_type_val);
                    count_query = count_query.bind(storage_type_val);
                }

                // LIMIT と OFFSET
                query = query.bind(limit).bind(offset);

                let rows = query.fetch_all(pool).await?;
                let items: Vec<Item> = rows.into_iter()
                    .map(|row| self.row_to_item_postgres(row))
                    .collect();

                let count_row = count_query.fetch_one(pool).await?;
                let total: i64 = count_row.get("count");

                Ok(ItemsListResponse {
                    items,
                    total,
                    page,
                    per_page,
                })
            }
            DatabasePool::Sqlite(pool) => {
                // 動的WHEREクエリを構築（簡単な方法）
                let mut where_conditions = Vec::new();

                // 検索条件
                if search.is_some() {
                    where_conditions.push("(name LIKE ? OR label_id LIKE ? OR model_number LIKE ? OR remarks LIKE ?)".to_string());
                }

                // 貸出状態フィルター
                if is_on_loan.is_some() {
                    where_conditions.push("is_on_loan = ?".to_string());
                }

                // 廃棄状態フィルター
                if is_disposed.is_some() {
                    where_conditions.push("is_disposed = ?".to_string());
                }

                // コンテナIDフィルター
                if container_id.is_some() {
                    where_conditions.push("container_id = ?".to_string());
                }

                // 保管タイプフィルター
                if storage_type.is_some() {
                    where_conditions.push("storage_type = ?".to_string());
                }

                let where_clause = if where_conditions.is_empty() {
                    String::new()
                } else {
                    format!("WHERE {}", where_conditions.join(" AND "))
                };

                // シンプルなアプローチで実装（フィルター条件ごとに分岐）
                let (items, total) = if search.is_none() && is_on_loan.is_none() && is_disposed.is_none() && container_id.is_none() && storage_type.is_none() {
                    // フィルターなし
                    let rows = sqlx::query(
                        r#"
                        SELECT 
                            id, name, label_id, model_number, remarks, purchase_year,
                            purchase_amount, durability_years, is_depreciation_target,
                            connection_names, cable_color_pattern, storage_location,
                            container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                            created_at, updated_at
                        FROM items 
                        ORDER BY created_at DESC 
                        LIMIT ?1 OFFSET ?2
                        "#,
                    )
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?;

                    let items: Vec<Item> = rows.into_iter()
                        .map(|row| self.row_to_item(row))
                        .collect();

                    let count_row = sqlx::query("SELECT COUNT(*) as count FROM items")
                        .fetch_one(pool)
                        .await?;
                    let total: i64 = count_row.get("count");

                    (items, total)
                } else {
                    // フィルターあり - 動的クエリを使用
                    let query_str = format!(
                        r#"
                        SELECT 
                            id, name, label_id, model_number, remarks, purchase_year,
                            purchase_amount, durability_years, is_depreciation_target,
                            connection_names, cable_color_pattern, storage_location,
                            container_id, storage_type, is_on_loan, qr_code_type, is_disposed, image_url,
                            created_at, updated_at
                        FROM items 
                        {}
                        ORDER BY created_at DESC 
                        LIMIT ? OFFSET ?
                        "#,
                        where_clause
                    );

                    let count_query_str = format!("SELECT COUNT(*) as count FROM items {}", where_clause);

                    // パラメーターをバインドするためのヘルパー関数
                    let mut query = sqlx::query(&query_str);
                    let mut count_query = sqlx::query(&count_query_str);

                    // 検索条件
                    if let Some(search_term) = &search {
                        let search_pattern = format!("%{}%", search_term);
                        query = query.bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone());
                        count_query = count_query.bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern.clone()).bind(search_pattern);
                    }

                    // 貸出状態フィルター
                    if let Some(loan_status) = is_on_loan {
                        let loan_value = if loan_status { 1i32 } else { 0i32 };
                        query = query.bind(loan_value);
                        count_query = count_query.bind(loan_value);
                    }

                    // 廃棄状態フィルター
                    if let Some(disposed_status) = is_disposed {
                        let disposed_value = if disposed_status { 1i32 } else { 0i32 };
                        query = query.bind(disposed_value);
                        count_query = count_query.bind(disposed_value);
                    }

                    // コンテナIDフィルター
                    if let Some(container_id_val) = &container_id {
                        query = query.bind(container_id_val);
                        count_query = count_query.bind(container_id_val);
                    }

                    // 保管タイプフィルター
                    if let Some(storage_type_val) = &storage_type {
                        query = query.bind(storage_type_val);
                        count_query = count_query.bind(storage_type_val);
                    }

                    // LIMIT/OFFSETをバインド
                    query = query.bind(limit).bind(offset);

                    let rows = query.fetch_all(pool).await?;
                    let items: Vec<Item> = rows.into_iter()
                        .map(|row| self.row_to_item(row))
                        .collect();

                    let count_row = count_query.fetch_one(pool).await?;
                    let total: i64 = count_row.get("count");

                    (items, total)
                };

                Ok(ItemsListResponse {
                    items,
                    total,
                    page,
                    per_page,
                })
            }
        }
    }

    pub async fn update_item(&self, id: i64, req: UpdateItemRequest) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // まず物品が存在するかチェック
                let _existing_item = self.get_item(id).await?;

                // JSON配列フィールドをシリアライズ
                let connection_names_json = req.connection_names
                    .as_ref()
                    .map(|names| serde_json::to_string(names))
                    .transpose()
                    .map_err(|e| AppError::InternalServerError(format!("Failed to serialize connection_names: {}", e)))?;

                let cable_color_pattern_json = req.cable_color_pattern
                    .as_ref()
                    .map(|pattern| serde_json::to_string(pattern))
                    .transpose()
                    .map_err(|e| AppError::InternalServerError(format!("Failed to serialize cable_color_pattern: {}", e)))?;

                let storage_location = req.storage_location;

                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE items SET
                        name = COALESCE($2, name),
                        label_id = COALESCE($3, label_id),
                        model_number = COALESCE($4, model_number),
                        remarks = COALESCE($5, remarks),
                        purchase_year = COALESCE($6, purchase_year),
                        purchase_amount = COALESCE($7, purchase_amount),
                        durability_years = COALESCE($8, durability_years),
                        is_depreciation_target = COALESCE($9, is_depreciation_target),
                        connection_names = COALESCE($10, connection_names),
                        cable_color_pattern = COALESCE($11, cable_color_pattern),
                        storage_location = COALESCE($12, storage_location),
                        container_id = COALESCE($13, container_id),
                        storage_type = COALESCE($14, storage_type),
                        qr_code_type = COALESCE($15, qr_code_type),
                        image_url = COALESCE($16, image_url),
                        updated_at = $17
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.label_id)
                .bind(&req.model_number)
                .bind(&req.remarks)
                .bind(&req.purchase_year)
                .bind(&req.purchase_amount)
                .bind(&req.durability_years)
                .bind(&req.is_depreciation_target)
                .bind(&connection_names_json)
                .bind(&cable_color_pattern_json)
                .bind(&storage_location)
                .bind(&req.container_id)
                .bind(&req.storage_type)
                .bind(&req.qr_code_type)
                .bind(&req.image_url)
                .bind(now)
                .execute(pool)
                .await?;

                // 更新後の物品を取得して返す
                self.get_item(id).await
            }
            DatabasePool::Sqlite(pool) => {
                // まず物品が存在するかチェック
                let _existing_item = self.get_item(id).await?;

                // JSON配列フィールドをシリアライズ
                let connection_names_json = req.connection_names
                    .as_ref()
                    .map(|names| serde_json::to_string(names))
                    .transpose()
                    .map_err(|e| AppError::InternalServerError(format!("Failed to serialize connection_names: {}", e)))?;

                let cable_color_pattern_json = req.cable_color_pattern
                    .as_ref()
                    .map(|pattern| serde_json::to_string(pattern))
                    .transpose()
                    .map_err(|e| AppError::InternalServerError(format!("Failed to serialize cable_color_pattern: {}", e)))?;

                let storage_location = req.storage_location;

                let now = chrono::Utc::now();

                sqlx::query!(
                    r#"
                    UPDATE items SET
                        name = COALESCE(?2, name),
                        label_id = COALESCE(?3, label_id),
                        model_number = COALESCE(?4, model_number),
                        remarks = COALESCE(?5, remarks),
                        purchase_year = COALESCE(?6, purchase_year),
                        purchase_amount = COALESCE(?7, purchase_amount),
                        durability_years = COALESCE(?8, durability_years),
                        is_depreciation_target = COALESCE(?9, is_depreciation_target),
                        connection_names = COALESCE(?10, connection_names),
                        cable_color_pattern = COALESCE(?11, cable_color_pattern),
                        storage_location = COALESCE(?12, storage_location),
                        container_id = COALESCE(?13, container_id),
                        storage_type = COALESCE(?14, storage_type),
                        qr_code_type = COALESCE(?15, qr_code_type),
                        image_url = COALESCE(?16, image_url),
                        updated_at = ?17
                    WHERE id = ?1
                    "#,
                    id,
                    req.name,
                    req.label_id,
                    req.model_number,
                    req.remarks,
                    req.purchase_year,
                    req.purchase_amount,
                    req.durability_years,
                    req.is_depreciation_target,
                    connection_names_json,
                    cable_color_pattern_json,
                    storage_location,
                    req.container_id,
                    req.storage_type,
                    req.qr_code_type,
                    req.image_url,
                    now
                )
                .execute(pool)
                .await?;

                // 更新後の物品を取得して返す
                self.get_item(id).await
            }
        }
    }

    pub async fn delete_item(&self, id: i64) -> AppResult<()> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // まず物品が存在し、貸出中でないかチェック
                let item = self.get_item(id).await?;
                
                if item.is_on_loan.unwrap_or(false) {
                    return Err(AppError::BadRequest("Cannot delete item that is currently on loan".to_string()));
                }

                // アクティブな貸出がないかチェック
                let active_loans = sqlx::query(
                    "SELECT COUNT(*) as count FROM loans WHERE item_id = $1 AND return_date IS NULL"
                )
                .bind(id)
                .fetch_one(pool)
                .await?;

                let count: i64 = active_loans.get("count");
                if count > 0 {
                    return Err(AppError::BadRequest("Cannot delete item with active loans".to_string()));
                }

                let result = sqlx::query("DELETE FROM items WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }
                Ok(())
            }
            DatabasePool::Sqlite(pool) => {
                // まず物品が存在し、貸出中でないかチェック
                let item = self.get_item(id).await?;
                
                if item.is_on_loan.unwrap_or(false) {
                    return Err(AppError::BadRequest("Cannot delete item that is currently on loan".to_string()));
                }

                // アクティブな貸出がないかチェック
                let active_loans = sqlx::query!(
                    "SELECT COUNT(*) as count FROM loans WHERE item_id = ?1 AND return_date IS NULL",
                    id
                )
                .fetch_one(pool)
                .await?;

                if active_loans.count > 0 {
                    return Err(AppError::BadRequest("Cannot delete item with active loans".to_string()));
                }

                let result = sqlx::query("DELETE FROM items WHERE id = ?1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }
                Ok(())
            }
        }
    }

    pub async fn dispose_item(&self, id: i64) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let now = Utc::now();
                let result = sqlx::query("UPDATE items SET is_disposed = true, updated_at = $2 WHERE id = $1")
                    .bind(id)
                    .bind(now)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }

                self.get_item(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let now = Utc::now();
                let result = sqlx::query("UPDATE items SET is_disposed = 1, updated_at = ?2 WHERE id = ?1")
                    .bind(id)
                    .bind(now)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }

                self.get_item(id).await
            }
        }
    }

    pub async fn undispose_item(&self, id: i64) -> AppResult<Item> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let now = Utc::now();
                let result = sqlx::query("UPDATE items SET is_disposed = false, updated_at = $2 WHERE id = $1")
                    .bind(id)
                    .bind(now)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }

                self.get_item(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let now = Utc::now();
                let result = sqlx::query("UPDATE items SET is_disposed = 0, updated_at = ?2 WHERE id = ?1")
                    .bind(id)
                    .bind(now)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Item with id {} not found", id)));
                }

                self.get_item(id).await
            }
        }
    }

    pub async fn get_connection_names_suggestions(&self) -> AppResult<Vec<String>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query("SELECT DISTINCT connection_names FROM items WHERE connection_names IS NOT NULL AND connection_names != ''")
                    .fetch_all(pool)
                    .await?;

                let mut suggestions = Vec::new();
                for row in rows {
                    if let Some(connection_names_str) = row.get::<Option<String>, _>("connection_names") {
                        if let Ok(names) = serde_json::from_str::<Vec<String>>(&connection_names_str) {
                            suggestions.extend(names);
                        }
                    }
                }

                // 重複を除去してソート
                suggestions.sort();
                suggestions.dedup();
                Ok(suggestions)
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query("SELECT DISTINCT connection_names FROM items WHERE connection_names IS NOT NULL AND connection_names != ''")
                    .fetch_all(pool)
                    .await?;

                let mut suggestions = Vec::new();
                for row in rows {
                    if let Some(json_str) = row.get::<Option<String>, _>("connection_names") {
                        if let Ok(names) = serde_json::from_str::<Vec<String>>(&json_str) {
                            suggestions.extend(names);
                        }
                    }
                }

                // 重複を除去してソート
                suggestions.sort();
                suggestions.dedup();
                Ok(suggestions)
            }
        }
    }

    pub async fn get_storage_locations_suggestions(&self) -> AppResult<Vec<String>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query("SELECT DISTINCT storage_location FROM items WHERE storage_location IS NOT NULL AND storage_location != ''")
                    .fetch_all(pool)
                    .await?;

                let mut suggestions = Vec::new();
                for row in rows {
                    if let Some(location_str) = row.get::<Option<String>, _>("storage_location") {
                        suggestions.push(location_str);
                    }
                }

                // 重複を除去してソート
                suggestions.sort();
                suggestions.dedup();
                Ok(suggestions)
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query("SELECT DISTINCT storage_location FROM items WHERE storage_location IS NOT NULL AND storage_location != ''")
                    .fetch_all(pool)
                    .await?;

                let mut suggestions = Vec::new();
                for row in rows {
                    if let Some(location_str) = row.get::<Option<String>, _>("storage_location") {
                        suggestions.push(location_str);
                    }
                }

                // 重複を除去してソート
                suggestions.sort();
                suggestions.dedup();
                Ok(suggestions)
            }
        }
    }

    pub async fn generate_label_ids(&self, quantity: u32) -> AppResult<Vec<String>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Get current counter value and increment it atomically
                let mut tx = pool.begin().await?;
                
                // Get current counter value
                let counter_row = sqlx::query("SELECT current_value FROM label_counter WHERE id = 1")
                    .fetch_one(&mut *tx)
                    .await?;
                let current_value: i64 = counter_row.get("current_value");
                
                // Check we have enough available IDs (max ZZZZ = 1,679,615)
                if current_value + quantity as i64 > 1679615 {
                    return Err(AppError::BadRequest("Not enough label IDs available".to_string()));
                }

                // Generate new label IDs in base-36 format
                let mut label_ids = Vec::new();
                for i in 1..=quantity {
                    let new_number = current_value + i as i64;
                    // Convert to base-36 and pad to 4 characters
                    let label_id = format!("{:0>4}", radix_fmt::radix_36(new_number as u32).to_string().to_uppercase());
                    label_ids.push(label_id);
                }
                
                // Update counter
                let new_counter_value = current_value + quantity as i64;
                sqlx::query("UPDATE label_counter SET current_value = $1 WHERE id = 1")
                    .bind(new_counter_value)
                    .execute(&mut *tx)
                    .await?;
                
                tx.commit().await?;

                Ok(label_ids)
            }
            DatabasePool::Sqlite(pool) => {
                // Get current counter value and increment it atomically
                let mut tx = pool.begin().await?;
                
                // Get current counter value
                let counter_row = sqlx::query("SELECT current_value FROM label_counter WHERE id = 1")
                    .fetch_one(&mut *tx)
                    .await?;
                let current_value: i64 = counter_row.get("current_value");
                
                // Check we have enough available IDs (max ZZZZ = 1,679,615)
                if current_value + quantity as i64 > 1679615 {
                    return Err(AppError::BadRequest("Not enough label IDs available".to_string()));
                }

                // Generate new label IDs in base-36 format
                let mut label_ids = Vec::new();
                for i in 1..=quantity {
                    let new_number = current_value + i as i64;
                    // Convert to base-36 and pad to 4 characters
                    let label_id = format!("{:0>4}", radix_fmt::radix_36(new_number as u32).to_string().to_uppercase());
                    label_ids.push(label_id);
                }
                
                // Update counter
                let new_counter_value = current_value + quantity as i64;
                sqlx::query("UPDATE label_counter SET current_value = ? WHERE id = 1")
                    .bind(new_counter_value)
                    .execute(&mut *tx)
                    .await?;
                
                tx.commit().await?;

                Ok(label_ids)
            }
        }
    }

    pub async fn get_all_labels(&self) -> AppResult<Vec<crate::handlers::labels::LabelInfo>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Get all possible 4-digit base-36 labels from 0000 to ZZZZ
                let mut all_labels = Vec::new();
                
                // Get used labels from database
                let used_labels_rows = sqlx::query("SELECT label_id, name FROM items WHERE LENGTH(label_id) = 4")
                    .fetch_all(pool)
                    .await?;
                
                let mut used_labels_map = std::collections::HashMap::new();
                for row in used_labels_rows {
                    let label_id: String = row.get("label_id");
                    let name: String = row.get("name");
                    used_labels_map.insert(label_id, name);
                }

                // Generate all possible labels and check if they're used (0 to ZZZZ in base-36)
                for i in 0..=1679615u32 {  // 36^4 - 1
                    let label_id = format!("{:0>4}", radix_fmt::radix_36(i).to_string().to_uppercase());
                    let label_info = crate::handlers::labels::LabelInfo {
                        id: label_id.clone(),
                        used: used_labels_map.contains_key(&label_id),
                        item_name: used_labels_map.get(&label_id).cloned(),
                    };
                    all_labels.push(label_info);
                }

                Ok(all_labels)
            }
            DatabasePool::Sqlite(pool) => {
                // Get all possible 4-digit base-36 labels from 0000 to ZZZZ
                let mut all_labels = Vec::new();
                
                // Get used labels from database
                let used_labels_rows = sqlx::query("SELECT label_id, name FROM items WHERE LENGTH(label_id) = 4")
                    .fetch_all(pool)
                    .await?;
                
                let mut used_labels_map = std::collections::HashMap::new();
                for row in used_labels_rows {
                    let label_id: String = row.get("label_id");
                    let name: String = row.get("name");
                    used_labels_map.insert(label_id, name);
                }

                // Generate all possible labels and check if they're used (0 to ZZZZ in base-36)
                for i in 0..=1679615u32 {  // 36^4 - 1
                    let label_id = format!("{:0>4}", radix_fmt::radix_36(i).to_string().to_uppercase());
                    let label_info = crate::handlers::labels::LabelInfo {
                        id: label_id.clone(),
                        used: used_labels_map.contains_key(&label_id),
                        item_name: used_labels_map.get(&label_id).cloned(),
                    };
                    all_labels.push(label_info);
                }

                Ok(all_labels)
            }
        }
    }

    fn row_to_item(&self, row: sqlx::sqlite::SqliteRow) -> Item {
        let connection_names: Option<Vec<String>> = row.get::<Option<String>, _>("connection_names")
            .and_then(|s| serde_json::from_str(&s).ok());
        let cable_color_pattern: Option<Vec<String>> = row.get::<Option<String>, _>("cable_color_pattern")
            .and_then(|s| serde_json::from_str(&s).ok());
        let storage_location: Option<String> = row.get::<Option<String>, _>("storage_location");

        Item {
            id: row.get("id"),
            name: row.get("name"),
            label_id: row.get("label_id"),
            model_number: row.get("model_number"),
            remarks: row.get("remarks"),
            purchase_year: row.get("purchase_year"),
            purchase_amount: row.get("purchase_amount"),
            durability_years: row.get("durability_years"),
            is_depreciation_target: row.get("is_depreciation_target"),
            connection_names,
            cable_color_pattern,
            storage_location,
            container_id: row.get("container_id"),
            storage_type: row.get::<Option<String>, _>("storage_type").unwrap_or_else(|| "location".to_string()),
            is_on_loan: row.get("is_on_loan"),
            qr_code_type: row.get("qr_code_type"),
            is_disposed: row.get("is_disposed"),
            image_url: row.get("image_url"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_item_postgres(&self, row: sqlx::postgres::PgRow) -> Item {
        let connection_names: Option<Vec<String>> = row.get::<Option<String>, _>("connection_names")
            .and_then(|s| serde_json::from_str(&s).ok());
        let cable_color_pattern: Option<Vec<String>> = row.get::<Option<String>, _>("cable_color_pattern")
            .and_then(|s| serde_json::from_str(&s).ok());
        let storage_location: Option<String> = row.get::<Option<String>, _>("storage_location");

        Item {
            id: row.get("id"),
            name: row.get("name"),
            label_id: row.get("label_id"),
            model_number: row.get("model_number"),
            remarks: row.get("remarks"),
            purchase_year: row.get("purchase_year"),
            purchase_amount: row.get("purchase_amount"),
            durability_years: row.get("durability_years"),
            is_depreciation_target: row.get("is_depreciation_target"),
            connection_names,
            cable_color_pattern,
            storage_location,
            container_id: row.get("container_id"),
            storage_type: row.get::<Option<String>, _>("storage_type").unwrap_or_else(|| "location".to_string()),
            is_on_loan: row.get("is_on_loan"),
            qr_code_type: row.get("qr_code_type"),
            is_disposed: row.get("is_disposed"),
            image_url: row.get("image_url"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}