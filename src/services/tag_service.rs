use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateTagRequest, Tag, TagsListResponse, UpdateTagRequest};
use sqlx::Row;

pub struct TagService {
    db: DatabasePool,
}

impl TagService {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn create_tag(&self, req: CreateTagRequest) -> AppResult<Tag> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO tags (name, color, description)
                    VALUES ($1, $2, $3)
                    RETURNING id
                    "#,
                )
                .bind(&req.name)
                .bind(&req.color)
                .bind(&req.description)
                .fetch_one(pool)
                .await?;

                let id: i64 = result.get("id");
                self.get_tag(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query(
                    r#"
                    INSERT INTO tags (name, color, description)
                    VALUES (?1, ?2, ?3)
                    "#,
                )
                .bind(&req.name)
                .bind(&req.color)
                .bind(&req.description)
                .execute(pool)
                .await?;

                let id = result.last_insert_rowid();
                self.get_tag(id).await
            }
        }
    }

    pub async fn get_tag(&self, id: i64) -> AppResult<Tag> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, color, description, created_at, updated_at
                    FROM tags
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", id)))?;

                Ok(self.row_to_tag_postgres(row))
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, name, color, description, created_at, updated_at
                    FROM tags
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", id)))?;

                Ok(self.row_to_tag(row))
            }
        }
    }

    pub async fn list_tags(&self, page: u32, per_page: u32) -> AppResult<TagsListResponse> {
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, color, description, created_at, updated_at
                    FROM tags
                    ORDER BY name ASC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let tags: Vec<Tag> = rows
                    .into_iter()
                    .map(|row| self.row_to_tag_postgres(row))
                    .collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM tags")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(TagsListResponse {
                    tags,
                    total,
                    page,
                    per_page,
                })
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT id, name, color, description, created_at, updated_at
                    FROM tags
                    ORDER BY name ASC
                    LIMIT ?1 OFFSET ?2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let tags: Vec<Tag> = rows.into_iter().map(|row| self.row_to_tag(row)).collect();

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM tags")
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.get("count");

                Ok(TagsListResponse {
                    tags,
                    total,
                    page,
                    per_page,
                })
            }
        }
    }

    pub async fn update_tag(&self, id: i64, req: UpdateTagRequest) -> AppResult<Tag> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let _existing = self.get_tag(id).await?;
                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE tags SET
                        name = COALESCE($2, name),
                        color = COALESCE($3, color),
                        description = COALESCE($4, description),
                        updated_at = $5
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.color)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_tag(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let _existing = self.get_tag(id).await?;
                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    UPDATE tags SET
                        name = COALESCE(?2, name),
                        color = COALESCE(?3, color),
                        description = COALESCE(?4, description),
                        updated_at = ?5
                    WHERE id = ?1
                    "#,
                )
                .bind(id)
                .bind(&req.name)
                .bind(&req.color)
                .bind(&req.description)
                .bind(now)
                .execute(pool)
                .await?;

                self.get_tag(id).await
            }
        }
    }

    pub async fn delete_tag(&self, id: i64) -> AppResult<()> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result = sqlx::query("DELETE FROM tags WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Tag with id {} not found", id)));
                }
                Ok(())
            }
            DatabasePool::Sqlite(pool) => {
                let result = sqlx::query("DELETE FROM tags WHERE id = ?1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound(format!("Tag with id {} not found", id)));
                }
                Ok(())
            }
        }
    }

    // Item-tag association methods
    pub async fn get_item_tags(&self, item_id: &str) -> AppResult<Vec<Tag>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT t.id, t.name, t.color, t.description, t.created_at, t.updated_at
                    FROM tags t
                    INNER JOIN item_tags it ON t.id = it.tag_id
                    WHERE it.item_id = $1::uuid
                    ORDER BY t.name ASC
                    "#,
                )
                .bind(item_id)
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|row| self.row_to_tag_postgres(row))
                    .collect())
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    r#"
                    SELECT t.id, t.name, t.color, t.description, t.created_at, t.updated_at
                    FROM tags t
                    INNER JOIN item_tags it ON t.id = it.tag_id
                    WHERE it.item_id = ?1
                    ORDER BY t.name ASC
                    "#,
                )
                .bind(item_id)
                .fetch_all(pool)
                .await?;

                Ok(rows.into_iter().map(|row| self.row_to_tag(row)).collect())
            }
        }
    }

    pub async fn set_item_tags(&self, item_id: &str, tag_ids: Vec<i64>) -> AppResult<Vec<Tag>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Delete existing tags
                sqlx::query("DELETE FROM item_tags WHERE item_id = $1::uuid")
                    .bind(item_id)
                    .execute(pool)
                    .await?;

                // Insert new tags
                for tag_id in &tag_ids {
                    sqlx::query("INSERT INTO item_tags (item_id, tag_id) VALUES ($1::uuid, $2)")
                        .bind(item_id)
                        .bind(tag_id)
                        .execute(pool)
                        .await?;
                }

                self.get_item_tags(item_id).await
            }
            DatabasePool::Sqlite(pool) => {
                // Delete existing tags
                sqlx::query("DELETE FROM item_tags WHERE item_id = ?1")
                    .bind(item_id)
                    .execute(pool)
                    .await?;

                // Insert new tags
                for tag_id in &tag_ids {
                    sqlx::query("INSERT INTO item_tags (item_id, tag_id) VALUES (?1, ?2)")
                        .bind(item_id)
                        .bind(tag_id)
                        .execute(pool)
                        .await?;
                }

                self.get_item_tags(item_id).await
            }
        }
    }

    fn row_to_tag(&self, row: sqlx::sqlite::SqliteRow) -> Tag {
        Tag {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_tag_postgres(&self, row: sqlx::postgres::PgRow) -> Tag {
        Tag {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
