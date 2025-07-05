use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: StorageType,
    pub local: Option<LocalStorageConfig>,
    pub s3: Option<S3Config>,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
}

fn default_max_file_size() -> u64 {
    5 // Default 5MB
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Local,
    S3,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocalStorageConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Config {
    pub bucket_name: String,
    pub region: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = ConfigBuilder::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default").required(false))
            // Add in the current environment file
            // Default to 'development' env
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of HYPERDASHI)
            // Eg.. `HYPERDASHI_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("HYPERDASHI").separator("_"))
            // You can override settings from env variables
            .add_source(
                Environment::default()
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" ")
            )
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://hyperdashi.db".to_string());
        
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        let storage_type = env::var("STORAGE_TYPE")
            .unwrap_or_else(|_| "local".to_string());

        let max_file_size_mb = env::var("STORAGE_MAX_FILE_SIZE_MB")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let storage = match storage_type.to_lowercase().as_str() {
            "s3" => {
                let bucket_name = env::var("S3_BUCKET_NAME")
                    .map_err(|_| ConfigError::Message("S3_BUCKET_NAME not set".to_string()))?;
                let region = env::var("AWS_REGION")
                    .map_err(|_| ConfigError::Message("AWS_REGION not set".to_string()))?;
                let access_key_id = env::var("AWS_ACCESS_KEY_ID").ok();
                let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").ok();

                StorageConfig {
                    storage_type: StorageType::S3,
                    local: None,
                    s3: Some(S3Config {
                        bucket_name,
                        region,
                        access_key_id,
                        secret_access_key,
                    }),
                    max_file_size_mb,
                }
            }
            _ => {
                let path = env::var("LOCAL_STORAGE_PATH")
                    .unwrap_or_else(|_| "./uploads".to_string());

                StorageConfig {
                    storage_type: StorageType::Local,
                    local: Some(LocalStorageConfig { path }),
                    s3: None,
                    max_file_size_mb,
                }
            }
        };

        Ok(Config {
            database: DatabaseConfig { url: database_url },
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            storage,
        })
    }
}