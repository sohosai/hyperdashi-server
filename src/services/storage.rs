use aws_sdk_s3::Client as S3Client;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

use crate::config::{Config, StorageType};
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub enum StorageService {
    S3(S3Storage),
    Local(LocalStorage),
}

impl StorageService {
    pub async fn new(config: &Config) -> AppResult<Self> {
        match &config.storage.storage_type {
            StorageType::S3 => Ok(StorageService::S3(S3Storage::new(config).await?)),
            StorageType::Local => Ok(StorageService::Local(LocalStorage::new(config)?)),
        }
    }

    pub async fn upload(&self, data: Vec<u8>, filename: &str, content_type: &str) -> AppResult<String> {
        match self {
            StorageService::S3(storage) => storage.upload(data, filename, content_type).await,
            StorageService::Local(storage) => storage.upload(data, filename, content_type).await,
        }
    }

    pub async fn delete(&self, url: &str) -> AppResult<()> {
        match self {
            StorageService::S3(storage) => storage.delete(url).await,
            StorageService::Local(storage) => storage.delete(url).await,
        }
    }

    pub fn get_url(&self, key: &str) -> String {
        match self {
            StorageService::S3(storage) => storage.get_url(key),
            StorageService::Local(storage) => storage.get_url(key),
        }
    }
}

#[derive(Clone)]
pub struct S3Storage {
    client: S3Client,
    bucket_name: String,
    base_url: String,
}

impl S3Storage {
    pub async fn new(config: &Config) -> AppResult<Self> {
        let s3_config = config.storage.s3.as_ref()
            .ok_or_else(|| AppError::ConfigError(
                config::ConfigError::Message("S3 configuration not found".to_string())
            ))?;

        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
        let client = S3Client::new(&aws_config);
        
        let base_url = format!("https://{}.s3.{}.amazonaws.com", 
            s3_config.bucket_name, 
            s3_config.region
        );

        Ok(Self {
            client,
            bucket_name: s3_config.bucket_name.clone(),
            base_url,
        })
    }

    pub async fn upload(&self, data: Vec<u8>, filename: &str, content_type: &str) -> AppResult<String> {
        let key = format!("images/{}/{}", Uuid::new_v4(), filename);
        
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to upload to S3: {}", e)))?;

        Ok(self.get_url(&key))
    }

    pub async fn delete(&self, url: &str) -> AppResult<()> {
        // Extract key from URL
        let key = url.strip_prefix(&format!("{}/", self.base_url))
            .ok_or_else(|| AppError::StorageError("Invalid S3 URL".to_string()))?;

        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to delete from S3: {}", e)))?;

        Ok(())
    }

    pub fn get_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url, key)
    }
}

#[derive(Clone)]
pub struct LocalStorage {
    base_path: PathBuf,
    base_url: String,
}

impl LocalStorage {
    pub fn new(config: &Config) -> AppResult<Self> {
        let local_config = config.storage.local.as_ref()
            .ok_or_else(|| AppError::ConfigError(
                config::ConfigError::Message("Local storage configuration not found".to_string())
            ))?;

        let base_path = PathBuf::from(&local_config.path);
        let base_url = format!("http://{}:{}/uploads", 
            config.server.host, 
            config.server.port
        );

        Ok(Self {
            base_path,
            base_url,
        })
    }

    async fn ensure_directory(&self, path: &Path) -> AppResult<()> {
        if !path.exists() {
            fs::create_dir_all(path).await?;
        }
        Ok(())
    }

    pub async fn upload(&self, data: Vec<u8>, filename: &str, _content_type: &str) -> AppResult<String> {
        let dir_name = Uuid::new_v4().to_string();
        let dir_path = self.base_path.join(&dir_name);
        
        self.ensure_directory(&dir_path).await?;
        
        let file_path = dir_path.join(filename);
        fs::write(&file_path, data).await?;

        let relative_path = format!("{}/{}", dir_name, filename);
        Ok(self.get_url(&relative_path))
    }

    pub async fn delete(&self, url: &str) -> AppResult<()> {
        // Extract relative path from URL
        let relative_path = url.strip_prefix(&format!("{}/", self.base_url))
            .ok_or_else(|| AppError::StorageError("Invalid local storage URL".to_string()))?;

        let file_path = self.base_path.join(relative_path);
        
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
            
            // Try to remove empty parent directory
            if let Some(parent) = file_path.parent() {
                let _ = fs::remove_dir(parent).await;
            }
        }

        Ok(())
    }

    pub fn get_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url, key)
    }
}