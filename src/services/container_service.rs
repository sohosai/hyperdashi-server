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
                    INSERT INTO containers (id, name, description, location, image_url, created_at, updated_at, is_disposed)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                )
                .bind(&container_id)
                .bind(&request.name)
                .bind(&request.description)
                .bind(&request.location)
                .bind(&request.image_url)
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

                let result = sqlx::query(
                    r#"
                    INSERT INTO containers (id, name, description, location, image_url, created_at, updated_at, is_disposed)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    "#
                )
                .bind(&container_id)
                .bind(&request.name)
                .bind(&request.description)
                .bind(&request.location)
                .bind(&request.image_url)
                .bind(now)
                .bind(now)
                .bind(false)
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
                    image_url: request.image_url,
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
                    "SELECT id, name, description, location, image_url, created_at, updated_at, is_disposed FROM containers WHERE id = $1"
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
                        image_url: row.get("image_url"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        is_disposed: row.get("is_disposed"),
                    }),
                    None => Err(AppError::NotFound("Container not found".to_string())),
                }
            }
            DatabasePool::Sqlite(pool) => {
                let row = sqlx::query(
                    "SELECT id, name, description, location, image_url, created_at, updated_at, is_disposed FROM containers WHERE id = ?"
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
                        image_url: row.get("image_url"),
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
        search: Option<&str>,
        sort_by: &str,
        sort_order: &str,
    ) -> AppResult<Vec<ContainerWithItemCount>> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let mut query_str = String::from(
                    r#"
                    SELECT
                        c.id, c.name, c.description, c.location, c.image_url, c.created_at, c.updated_at, c.is_disposed,
                        COUNT(i.id) as item_count
                    FROM containers c
                    LEFT JOIN items i ON c.id = i.container_id AND i.storage_type = 'container' AND (i.is_disposed IS NULL OR i.is_disposed = false)
                    WHERE 1=1
                    "#,
                );

                let mut param_index = 1;

                if !include_disposed {
                    query_str.push_str(" AND c.is_disposed = false");
                }

                if location_filter.is_some() {
                    query_str.push_str(&format!(" AND c.location = ${}", param_index));
                    param_index += 1;
                }

                if search.is_some() {
                    let search_clause = format!(
                        " AND (c.name ILIKE ${} OR c.description ILIKE ${} OR c.location ILIKE ${})",
                        param_index, param_index + 1, param_index + 2
                    );
                    query_str.push_str(&search_clause);
                }

                query_str.push_str(" GROUP BY c.id, c.name, c.description, c.location, c.image_url, c.created_at, c.updated_at, c.is_disposed");

                let sort_column = match sort_by {
                    "name" => "c.name",
                    "location" => "c.location",
                    "item_count" => "item_count",
                    "created_at" => "c.created_at",
                    "updated_at" => "c.updated_at",
                    "is_disposed" => "c.is_disposed",
                    _ => "c.created_at",
                };
                let sort_direction = if sort_order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };
                query_str.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

                let mut query = sqlx::query(&query_str);

                if let Some(location) = location_filter {
                    query = query.bind(location);
                }

                if let Some(search_term) = search {
                    let search_param = format!("%{}%", search_term);
                    query = query.bind(search_param.clone()).bind(search_param.clone()).bind(search_param);
                }

                let rows = query.fetch_all(pool).await?;

                let containers = rows
                    .into_iter()
                    .map(|row| ContainerWithItemCount {
                        container: Container {
                            id: row.get("id"),
                            name: row.get("name"),
                            description: row.get("description"),
                            location: row.get("location"),
                            image_url: row.get("image_url"),
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
                        c.id, c.name, c.description, c.location, c.image_url, c.created_at, c.updated_at, c.is_disposed,
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

                if let Some(search_term) = search {
                    query.push_str(" AND (c.name LIKE ? OR c.description LIKE ? OR c.location LIKE ?)");
                    let search_param = format!("%{}%", search_term);
                    params.push(search_param.clone());
                    params.push(search_param.clone());
                    params.push(search_param);
                }

                query.push_str(" GROUP BY c.id, c.name, c.description, c.location, c.image_url, c.created_at, c.updated_at, c.is_disposed");

                let sort_column = match sort_by {
                    "name" => "c.name",
                    "location" => "c.location",
                    "item_count" => "item_count",
                    "created_at" => "c.created_at",
                    "updated_at" => "c.updated_at",
                    "is_disposed" => "c.is_disposed",
                    _ => "c.created_at",
                };
                let sort_direction = if sort_order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };
                query.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

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
                                image_url: row.get("image_url"),
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

                if request.image_url.is_some() {
                    updates.push(format!("image_url = ${}", param_index));
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

                if let Some(image_url) = &request.image_url {
                    query_builder = query_builder.bind(image_url);
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

                if let Some(image_url) = &request.image_url {
                    updates.push("image_url = ?");
                    params.push(image_url.clone());
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
                    "SELECT id, name, description, location, image_url, created_at, updated_at, is_disposed FROM containers WHERE location = $1 AND is_disposed = false ORDER BY name"
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
                        image_url: row.get("image_url"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                        is_disposed: row.get("is_disposed"),
                    })
                    .collect();

                Ok(containers)
            }
            DatabasePool::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT id, name, description, location, image_url, created_at, updated_at, is_disposed FROM containers WHERE location = ? AND is_disposed = 0 ORDER BY name"
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
                        image_url: row.get("image_url"),
                        created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc(),
                        updated_at: row.get::<chrono::NaiveDateTime, _>("updated_at").and_utc(),
                        is_disposed: row.get("is_disposed"),
                    })
                    .collect();

                Ok(containers)
            }
        }
    }

    pub async fn check_container_id_exists(&self, id: &str) -> AppResult<bool> {
        match &self.db {
            DatabasePool::Postgres(pool) => {
                let result: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM containers WHERE id = $1)")
                    .bind(id)
                    .fetch_one(pool)
                    .await?;
                Ok(result.0)
            }
            DatabasePool::Sqlite(pool) => {
                let result: (i32,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM containers WHERE id = ?)")
                    .bind(id)
                    .fetch_one(pool)
                    .await?;
                Ok(result.0 == 1)
            }
        }
    }

    pub async fn bulk_delete_containers(&self, ids: &[String]) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        // Check if any containers have items
        let has_items = self.check_containers_have_items(ids).await?;
        if has_items {
            return Err(AppError::BadRequest(
                "Cannot delete containers that contain items".to_string(),
            ));
        }

        match &self.db {
            DatabasePool::Postgres(pool) => {
                sqlx::query("DELETE FROM containers WHERE id = ANY($1)")
                    .bind(ids)
                    .execute(pool)
                    .await?;
            }
            DatabasePool::Sqlite(pool) => {
                let query = format!(
                    "DELETE FROM containers WHERE id IN ({})",
                    ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                );
                let mut query_builder = sqlx::query(&query);
                for id in ids {
                    query_builder = query_builder.bind(id);
                }
                query_builder.execute(pool).await?;
            }
        }
        Ok(())
    }

    pub async fn bulk_update_disposed_status(
        &self,
        ids: &[String],
        is_disposed: bool,
    ) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let now = chrono::Utc::now();

        match &self.db {
            DatabasePool::Postgres(pool) => {
                sqlx::query("UPDATE containers SET is_disposed = $1, updated_at = $2 WHERE id = ANY($3)")
                    .bind(is_disposed)
                    .bind(now)
                    .bind(ids)
                    .execute(pool)
                    .await?;
            }
            DatabasePool::Sqlite(pool) => {
                let query = format!(
                    "UPDATE containers SET is_disposed = ?, updated_at = ? WHERE id IN ({})",
                    ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                );
                let mut query_builder = sqlx::query(&query);
                query_builder = query_builder.bind(is_disposed);
                query_builder = query_builder.bind(now);
                for id in ids {
                    query_builder = query_builder.bind(id);
                }
                query_builder.execute(pool).await?;
            }
        }
        Ok(())
    }

    async fn check_containers_have_items(&self, ids: &[String]) -> AppResult<bool> {
        if ids.is_empty() {
            return Ok(false);
        }

        match &self.db {
            DatabasePool::Postgres(pool) => {
                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM items WHERE container_id = ANY($1) AND storage_type = 'container' AND (is_disposed IS NULL OR is_disposed = false)"
                )
                .bind(ids)
                .fetch_one(pool)
                .await?;
                Ok(count > 0)
            }
            DatabasePool::Sqlite(pool) => {
                let query = format!(
                    "SELECT COUNT(*) FROM items WHERE container_id IN ({}) AND storage_type = 'container' AND (is_disposed IS NULL OR is_disposed = 0)",
                    ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
                );
                let mut query_builder = sqlx::query_scalar(&query);
                for id in ids {
                    query_builder = query_builder.bind(id);
                }
                let count: i64 = query_builder.fetch_one(pool).await?;
                Ok(count > 0)
            }
        }
    }
}
