use crate::db::DatabasePool;
use crate::error::{AppError, AppResult};
use crate::models::{
    Container, ContainerWithItemCount, CreateContainerRequest, UpdateContainerRequest,
};
use crate::services::item_service::ItemService;
use sqlx::Row;

pub struct ContainerService {
    db: DatabasePool,
    item_service: ItemService,
}

impl ContainerService {
    pub fn new(db: DatabasePool) -> Self {
        let item_service = ItemService::new(db.clone());
        Self { db, item_service }
    }

    pub async fn create_container(&self, request: CreateContainerRequest) -> AppResult<Container> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Generate container ID using the same method as items
                let container_ids = self.item_service.generate_label_ids(1).await?;
                let container_id = container_ids.into_iter().next().ok_or_else(|| {
                    AppError::InternalServerError("Failed to generate container ID".to_string())
                })?;

                let now = chrono::Utc::now();

                sqlx::query(
                    r#"
                    INSERT INTO containers (id, name, description, location, created_at, updated_at, is_disposed)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(&container_id)
                .bind(&request.name)
                .bind(&request.description)
                .bind(&request.location)
                .bind(now)
                .bind(now)
                .bind(false)
                .execute(pool)
                .await?;

                // Return the created container
                self.get_container(&container_id).await
            }
            DatabasePool::Sqlite(pool) => {
                // Generate container ID using the same method as items
                let container_ids = self.item_service.generate_label_ids(1).await?;
                let container_id = container_ids.into_iter().next().ok_or_else(|| {
                    AppError::InternalServerError("Failed to generate container ID".to_string())
                })?;

                let now = chrono::Utc::now();

                let result = sqlx::query!(
                    r#"
                    INSERT INTO containers (id, name, description, location, created_at, updated_at, is_disposed)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                    container_id,
                    request.name,
                    request.description,
                    request.location,
                    now,
                    now,
                    false
                )
                .execute(pool)
                .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::InternalServerError(
                        "Failed to create container".to_string(),
                    ));
                }

                Ok(Container {
                    id: container_id,
                    name: request.name,
                    description: request.description,
                    location: request.location,
                    created_at: now,
                    updated_at: now,
                    is_disposed: false,
                })
            }
        }
    }

    pub async fn get_container(&self, id: &str) -> AppResult<Container> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT id, name, description, location, created_at, updated_at, is_disposed FROM containers WHERE id = $1"
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                match row {
                    Some(row) => Ok(Container {
                        id: row.get("id"),
                        name: row.get("name"),
                        description: row.get("description"),
                        location: row.get("location"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        is_disposed: row.get("is_disposed"),
                    }),
                    None => Err(AppError::NotFound("Container not found".to_string())),
                }
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, name, description, location, created_at, updated_at, is_disposed FROM containers WHERE id = ?"
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;

                match row {
                    Some(row) => Ok(Container {
                        id: row.get("id"),
                        name: row.get("name"),
                        description: row.get("description"),
                        location: row.get("location"),
                        created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc(),
                        updated_at: row.get::<chrono::NaiveDateTime, _>("updated_at").and_utc(),
                        is_disposed: {
                            // Handle both TEXT and INTEGER types for is_disposed
                            if let Ok(int_val) = row.try_get::<Option<i32>, _>("is_disposed") {
                                int_val.unwrap_or(0) != 0
                            } else if let Ok(text_val) =
                                row.try_get::<Option<String>, _>("is_disposed")
                            {
                                matches!(
                                    text_val.as_deref(),
                                    Some("1") | Some("true") | Some("TRUE")
                                )
                            } else {
                                false
                            }
                        },
                    }),
                    None => Err(AppError::NotFound("Container not found".to_string())),
                }
            }
        }
    }

    pub async fn list_containers(
        &self,
        location_filter: Option<&str>,
        include_disposed: bool,
    ) -> AppResult<Vec<ContainerWithItemCount>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let mut query = String::from(
                    r#"
                    SELECT
                        c.id, c.name, c.description, c.location, c.created_at, c.updated_at, c.is_disposed,
                        COUNT(i.id) as item_count
                    FROM containers c
                    LEFT JOIN items i ON c.id = i.container_id AND i.storage_type = 'container' AND (i.is_disposed IS NULL OR i.is_disposed = false)
                    WHERE 1=1
                    "#,
                );

                let param_index = 1;

                if !include_disposed {
                    query.push_str(" AND c.is_disposed = false");
                }

                if location_filter.is_some() {
                    query.push_str(&format!(" AND c.location = ${}", param_index));
                }

                query.push_str(" GROUP BY c.id, c.name, c.description, c.location, c.created_at, c.updated_at, c.is_disposed");
                query.push_str(" ORDER BY c.created_at DESC");

                let mut sqlx_query = sqlx::query(&query);

                if let Some(location) = location_filter {
                    sqlx_query = sqlx_query.bind(location);
                }

                let rows = sqlx_query.fetch_all(pool).await?;

                let containers = rows
                    .into_iter()
                    .map(|row| ContainerWithItemCount {
                        container: Container {
                            id: row.get("id"),
                            name: row.get("name"),
                            description: row.get("description"),
                            location: row.get("location"),
                            created_at: row.get("created_at"),
                            updated_at: row.get("updated_at"),
                            is_disposed: row.get("is_disposed"),
                        },
                        item_count: row.get::<i64, _>("item_count"),
                    })
                    .collect();

                Ok(containers)
            }
            DatabasePool::Sqlite(pool) => {
                let mut query = String::from(
                    r#"
                    SELECT
                        c.id, c.name, c.description, c.location, c.created_at, c.updated_at, c.is_disposed,
                        COUNT(i.id) as item_count
                    FROM containers c
                    LEFT JOIN items i ON c.id = i.container_id AND i.storage_type = 'container' AND (i.is_disposed IS NULL OR i.is_disposed = 0)
                    WHERE 1=1
                    "#,
                );

                let mut params: Vec<String> = Vec::new();

                if !include_disposed {
                    query.push_str(" AND c.is_disposed = 0");
                }

                if let Some(location) = location_filter {
                    query.push_str(" AND c.location = ?");
                    params.push(location.to_string());
                }

                query.push_str(" GROUP BY c.id ORDER BY c.created_at DESC");

                let mut query_builder = sqlx::query(&query);
                for param in params {
                    query_builder = query_builder.bind(param);
                }

                let rows = query_builder.fetch_all(pool).await?;

                let containers = rows
                    .into_iter()
                    .map(|row| {
                        ContainerWithItemCount {
                            container: Container {
                                id: row.get::<Option<String>, _>("id").unwrap_or_default(),
                                name: row.get::<Option<String>, _>("name").unwrap_or_default(),
                                description: row.get("description"),
                                location: row
                                    .get::<Option<String>, _>("location")
                                    .unwrap_or_default(),
                                created_at: row
                                    .get::<Option<chrono::NaiveDateTime>, _>("created_at")
                                    .map(|dt| {
                                        chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)
                                    })
                                    .unwrap_or_default(),
                                updated_at: row
                                    .get::<Option<chrono::NaiveDateTime>, _>("updated_at")
                                    .map(|dt| {
                                        chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)
                                    })
                                    .unwrap_or_default(),
                                is_disposed: {
                                    // Handle both TEXT and INTEGER types for is_disposed
                                    if let Ok(int_val) =
                                        row.try_get::<Option<i32>, _>("is_disposed")
                                    {
                                        int_val.unwrap_or(0) != 0
                                    } else if let Ok(text_val) =
                                        row.try_get::<Option<String>, _>("is_disposed")
                                    {
                                        matches!(
                                            text_val.as_deref(),
                                            Some("1") | Some("true") | Some("TRUE")
                                        )
                                    } else {
                                        false
                                    }
                                },
                            },
                            item_count: row.get("item_count"),
                        }
                    })
                    .collect();

                Ok(containers)
            }
        }
    }

    pub async fn update_container(
        &self,
        id: &str,
        request: UpdateContainerRequest,
    ) -> AppResult<Container> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let mut updates = Vec::new();
                let mut param_index = 1;

                if request.name.is_some() {
                    updates.push(format!("name = ${}", param_index));
                    param_index += 1;
                }

                if request.description.is_some() {
                    updates.push(format!("description = ${}", param_index));
                    param_index += 1;
                }

                if request.location.is_some() {
                    updates.push(format!("location = ${}", param_index));
                    param_index += 1;
                }

                if request.is_disposed.is_some() {
                    updates.push(format!("is_disposed = ${}", param_index));
                    param_index += 1;
                }

                if updates.is_empty() {
                    return Err(AppError::BadRequest("No fields to update".to_string()));
                }

                updates.push(format!("updated_at = ${}", param_index));
                let now = chrono::Utc::now();
                param_index += 1;

                let query = format!(
                    "UPDATE containers SET {} WHERE id = ${}",
                    updates.join(", "),
                    param_index
                );

                let mut query_builder = sqlx::query(&query);

                if let Some(name) = &request.name {
                    query_builder = query_builder.bind(name);
                }

                if let Some(description) = &request.description {
                    query_builder = query_builder.bind(description);
                }

                if let Some(location) = &request.location {
                    query_builder = query_builder.bind(location);
                }

                if let Some(is_disposed) = request.is_disposed {
                    query_builder = query_builder.bind(is_disposed);
                }

                query_builder = query_builder.bind(now).bind(id);

                let result = query_builder.execute(pool).await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound("Container not found".to_string()));
                }

                self.get_container(id).await
            }
            DatabasePool::Sqlite(pool) => {
                let mut updates = Vec::new();
                let mut params: Vec<String> = Vec::new();

                if let Some(name) = &request.name {
                    updates.push("name = ?");
                    params.push(name.clone());
                }

                if let Some(description) = &request.description {
                    updates.push("description = ?");
                    params.push(description.clone());
                }

                if let Some(location) = &request.location {
                    updates.push("location = ?");
                    params.push(location.clone());
                }

                if let Some(is_disposed) = request.is_disposed {
                    updates.push("is_disposed = ?");
                    // Store as text to match the current database schema
                    params.push(if is_disposed {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    });
                }

                if updates.is_empty() {
                    return Err(AppError::BadRequest("No fields to update".to_string()));
                }

                updates.push("updated_at = ?");
                let now = chrono::Utc::now();
                params.push(now.to_rfc3339());
                params.push(id.to_string());

                let query = format!("UPDATE containers SET {} WHERE id = ?", updates.join(", "));

                let mut query_builder = sqlx::query(&query);
                for param in params {
                    query_builder = query_builder.bind(param);
                }

                let result = query_builder.execute(pool).await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound("Container not found".to_string()));
                }

                self.get_container(id).await
            }
        }
    }

    pub async fn delete_container(&self, id: &str) -> AppResult<()> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                // Check if container has items
                let items_count = sqlx::query(
                    "SELECT COUNT(*) as count FROM items WHERE container_id = $1 AND storage_type = 'container'"
                )
                .bind(id)
                .fetch_one(pool)
                .await?;

                let count: i64 = items_count.get("count");
                if count > 0 {
                    return Err(AppError::BadRequest(
                        "Cannot delete container that contains items".to_string(),
                    ));
                }

                let result = sqlx::query("DELETE FROM containers WHERE id = $1")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound("Container not found".to_string()));
                }

                Ok(())
            }
            DatabasePool::Sqlite(pool) => {
                // Check if container has items
                let item_count_row = sqlx::query(
                    "SELECT COUNT(*) as count FROM items WHERE container_id = ? AND storage_type = 'container' AND (is_disposed IS NULL OR is_disposed = 0)"
                )
                .bind(id)
                .fetch_one(pool)
                .await?;

                let item_count: i64 = item_count_row.get("count");
                if item_count > 0 {
                    return Err(AppError::BadRequest(
                        "Cannot delete container with items. Move or remove items first."
                            .to_string(),
                    ));
                }

                let result = sqlx::query("DELETE FROM containers WHERE id = ?")
                    .bind(id)
                    .execute(pool)
                    .await?;

                if result.rows_affected() == 0 {
                    return Err(AppError::NotFound("Container not found".to_string()));
                }

                Ok(())
            }
        }
    }

    pub async fn get_containers_by_location(&self, location: &str) -> AppResult<Vec<Container>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT id, name, description, location, created_at, updated_at, is_disposed FROM containers WHERE location = $1 AND is_disposed = false ORDER BY name"
                )
                .bind(location)
                .fetch_all(pool)
                .await?;

                let containers = rows
                    .into_iter()
                    .map(|row| Container {
                        id: row.get("id"),
                        name: row.get("name"),
                        description: row.get("description"),
                        location: row.get("location"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        is_disposed: row.get("is_disposed"),
                    })
                    .collect();

                Ok(containers)
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, name, description, location, created_at, updated_at, is_disposed FROM containers WHERE location = ? AND is_disposed = 0 ORDER BY name"
                )
                .bind(location)
                .fetch_all(pool)
                .await?;

                let containers = rows
                    .into_iter()
                    .map(|row| Container {
                        id: row.get("id"),
                        name: row.get("name"),
                        description: row.get("description"),
                        location: row.get("location"),
                        created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc(),
                        updated_at: row.get::<chrono::NaiveDateTime, _>("updated_at").and_utc(),
                        is_disposed: row.get("is_disposed"),
                    })
                    .collect();

                Ok(containers)
            }
        }
    }
}
