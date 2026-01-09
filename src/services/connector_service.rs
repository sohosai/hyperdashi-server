use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{
    Connector, ConnectorsListResponse, CreateConnectorRequest, UpdateConnectorRequest,
};
use sqlx::Row;

pub struct ConnectorService {
    db: DatabasePool,
}

impl ConnectorService {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_connector(&self, req: CreateConnectorRequest) -> AppResult<Connector> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO connectors (name, gender, description)
                    VALUES ($1, $2, $3)
                    RETURNING id
                    "#,
                )
                .bind(&req.name)
                .bind(&req.gender)
                .bind(&req.description)
                .fetch_one(pool)
                .await?;

                let id: i64 = result.get("id");
                self.get_connector(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO connectors (name, gender, description)
                    VALUES (?1, ?2, ?3)
                    "#,
                )
                .bind(&req.name)
                .bind(&req.gender)
                .bind(&req.description)
                .execute(pool)
                .await?;

                let id = result.last_insert_rowid();
                self.get_connector(id).await
            }
        }
    }

    pub async fn get_connector(&self, id: i64) -> AppResult<Connector> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, gender, description, created_at, updated_at
                    FROM connectors
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Connector with id {} not found", id)))?;

                Ok(self.row_to_connector_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, gender, description, created_at, updated_at
                    FROM connectors
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Connector with id {} not found", id)))?;

                Ok(self.row_to_connector(row))
            }
        }
    }

    pub async fn list_connectors(
        &self,
        page: u32,
        per_page: u32,
    ) -> AppResult<ConnectorsListResponse> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, gender, description, created_at, updated_at
                    FROM connectors
                    ORDER BY name ASC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let connectors: Vec<Connector> = rows
                    .into_iter()
                    .map(|row| self.row_to_connector_postgres(row))
                    .collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM connectors")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(ConnectorsListResponse {
                    connectors,
                    total,
                    page,
                    per_page,
                })
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, gender, description, created_at, updated_at
                    FROM connectors
                    ORDER BY name ASC
                    LIMIT ?1 OFFSET ?2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let connectors: Vec<Connector> = rows
                    .into_iter()
                    .map(|row| self.row_to_connector(row))
                    .collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM connectors")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(ConnectorsListResponse {
                    connectors,
                    total,
                    page,
                    per_page,
                })
            }
        }
    }

    pub async fn update_connector(
        &self,
        id: i64,
        req: UpdateConnectorRequest,
    ) -> AppResult<Connector> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let _existing = self.get_connector(id).await?;
                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE connectors SET
                        name = COALESCE($2, name),
                        gender = COALESCE($3, gender),
                        description = COALESCE($4, description),
                        updated_at = $5
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.gender)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_connector(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let _existing = self.get_connector(id).await?;
                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE connectors SET
                        name = COALESCE(?2, name),
                        gender = COALESCE(?3, gender),
                        description = COALESCE(?4, description),
                        updated_at = ?5
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.gender)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_connector(id).await
            }
        }
    }

    pub async fn delete_connector(&self, id: i64) -> AppResult<()> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query("DELETE FROM connectors WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!(
                        "Connector with id {} not found",
                        id
                    )));
                }
                Ok(())
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query("DELETE FROM connectors WHERE id = ?1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!(
                        "Connector with id {} not found",
                        id
                    )));
                }
                Ok(())
            }
        }
    }

    fn row_to_connector(&self, row: sqlx::sqlite::SqliteRow) -> Connector {
        Connector {
            id: row.get("id"),
            name: row.get("name"),
            gender: row.get("gender"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_connector_postgres(&self, row: sqlx::postgres::PgRow) -> Connector {
        Connector {
            id: row.get("id"),
            name: row.get("name"),
            gender: row.get("gender"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
