use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use sqlx::migrate::Migrator;

use crate::config::Config;
use crate::error::AppResult;

#[derive(Clone)]
pub enum DatabasePool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl DatabasePool {
    pub async fn new(config: &Config) -> AppResult<Self> {
        let database_url = &config.database.url;

        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .connect(database_url)
                .await?;

            Ok(DatabasePool::Postgres(pool))
        } else if database_url.starts_with("sqlite://") {
            let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

            let pool = SqlitePoolOptions::new()
                .max_connections(10)
                .connect_with(options)
                .await?;

            Ok(DatabasePool::Sqlite(pool))
        } else {
            Err(crate::error::AppError::ConfigError(
                config::ConfigError::Message(
                    "Invalid database URL. Must start with postgres:// or sqlite://".to_string(),
                ),
            ))
        }
    }

    pub async fn migrate(&self) -> AppResult<()> {
        match self {
            DatabasePool::Postgres(pool) => {
                Migrator::new(std::path::Path::new("./migrations/postgres")).await?
                    .run(pool)
                    .await?;
            }
            DatabasePool::Sqlite(pool) => {
                Migrator::new(std::path::Path::new("./migrations/sqlite")).await?
                    .run(pool)
                    .await?;
            }
        }
        Ok(())
    }

    pub fn postgres(&self) -> Option<&PgPool> {
        match self {
            DatabasePool::Postgres(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn sqlite(&self) -> Option<&SqlitePool> {
        match self {
            DatabasePool::Sqlite(pool) => Some(pool),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! query_as {
    ($query:expr, $pool:expr) => {
        match $pool {
            $crate::db::DatabasePool::Postgres(pool) => {
                sqlx::query_as($query).fetch_all(pool).await
            }
            $crate::db::DatabasePool::Sqlite(pool) => sqlx::query_as!($query).fetch_all(pool).await,
        }
    };
}
