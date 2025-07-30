use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{
    CableColor, CableColorsListResponse, CreateCableColorRequest, UpdateCableColorRequest,
};
use sqlx::Row;

pub struct CableColorService {
    db: DatabasePool,
}

impl CableColorService {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_cable_color(&self, req: CreateCableColorRequest) -> AppResult<CableColor> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO cable_colors (name, hex_code, description)
                    VALUES ($1, $2, $3)
                    RETURNING id
                    "#,
                )
                .bind(&req.name)
                .bind(&req.hex_code)
                .bind(&req.description)
                .fetch_one(pool)
                .await?;

                let id: i64 = result.get("id");
                self.get_cable_color(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO cable_colors (name, hex_code, description)
                    VALUES (?1, ?2, ?3)
                    "#,
                )
                .bind(&req.name)
                .bind(&req.hex_code)
                .bind(&req.description)
                .execute(pool)
                .await?;

                let id = result.last_insert_rowid();
                self.get_cable_color(id).await
            }
        }
    }

    pub async fn get_cable_color(&self, id: i64) -> AppResult<CableColor> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, hex_code, description, created_at, updated_at
                    FROM cable_colors
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Cable color with id {} not found", id))
                })?;

                Ok(self.row_to_cable_color_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, hex_code, description, created_at, updated_at
                    FROM cable_colors
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Cable color with id {} not found", id))
                })?;

                Ok(self.row_to_cable_color(row))
            }
        }
    }

    pub async fn list_cable_colors(
        &self,
        page: u32,
        per_page: u32,
    ) -> AppResult<CableColorsListResponse> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, hex_code, description, created_at, updated_at
                    FROM cable_colors
                    ORDER BY created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let cable_colors: Vec<CableColor> = rows
                    .into_iter()
                    .map(|row| self.row_to_cable_color_postgres(row))
                    .collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM cable_colors")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(CableColorsListResponse {
                    cable_colors,
                    total,
                    page,
                    per_page,
                })
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, hex_code, description, created_at, updated_at
                    FROM cable_colors
                    ORDER BY created_at DESC
                    LIMIT ?1 OFFSET ?2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let cable_colors: Vec<CableColor> = rows
                    .into_iter()
                    .map(|row| self.row_to_cable_color(row))
                    .collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM cable_colors")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(CableColorsListResponse {
                    cable_colors,
                    total,
                    page,
                    per_page,
                })
            }
        }
    }

    pub async fn update_cable_color(
        &self,
        id: i64,
        req: UpdateCableColorRequest,
    ) -> AppResult<CableColor> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // まず色が存在するかチェック
                let _existing_color = self.get_cable_color(id).await?;

                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE cable_colors SET
                        name = COALESCE($2, name),
                        hex_code = COALESCE($3, hex_code),
                        description = COALESCE($4, description),
                        updated_at = $5
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.hex_code)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_cable_color(id).await
            }
            DatabasePool::Sqlite(pool) => {
                // まず色が存在するかチェック
                let _existing_color = self.get_cable_color(id).await?;

                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE cable_colors SET
                        name = COALESCE(?2, name),
                        hex_code = COALESCE(?3, hex_code),
                        description = COALESCE(?4, description),
                        updated_at = ?5
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.hex_code)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_cable_color(id).await
            }
        }
    }

    pub async fn delete_cable_color(&self, id: i64) -> AppResult<()> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query("DELETE FROM cable_colors WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!(
                        "Cable color with id {} not found",
                        id
                    )));
                }

                Ok(())
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query("DELETE FROM cable_colors WHERE id = ?1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!(
                        "Cable color with id {} not found",
                        id
                    )));
                }
                Ok(())
            }
        }
    }

    fn row_to_cable_color(&self, row: sqlx::sqlite::SqliteRow) -> CableColor {
        CableColor {
            id: row.get("id"),
            name: row.get("name"),
            hex_code: row.get("hex_code"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_cable_color_postgres(&self, row: sqlx::postgres::PgRow) -> CableColor {
        CableColor {
            id: row.get("id"),
            name: row.get("name"),
            hex_code: row.get("hex_code"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
