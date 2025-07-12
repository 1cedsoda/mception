use super::config::ConfigStorage;
use crate::core::{ServerConfig, StorageError, MceptionResult, MceptionError};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use chrono::Utc;

/// File-based configuration storage implementation
#[derive(Debug, Clone)]
pub struct FileConfigStorage {
    config_path: String,
}

impl FileConfigStorage {
    pub fn new(config_path: impl Into<String>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }
    
    fn backup_path(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}.backup.{}", self.config_path, timestamp)
    }
}

#[async_trait]
impl ConfigStorage for FileConfigStorage {
    async fn load_config(&self) -> MceptionResult<ServerConfig> {
        if !Path::new(&self.config_path).exists() {
            return Ok(ServerConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(StorageError::from)?;
            
        let config: ServerConfig = serde_json::from_str(&content)
            .map_err(StorageError::from)?;
            
        Ok(config)
    }

    async fn save_config(&self, config: &ServerConfig) -> MceptionResult<()> {
        let content = serde_json::to_string_pretty(config)
            .map_err(StorageError::from)?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(StorageError::from)?;
        }
        
        fs::write(&self.config_path, content)
            .await
            .map_err(StorageError::from)?;
            
        Ok(())
    }
    
    async fn config_exists(&self) -> MceptionResult<bool> {
        Ok(Path::new(&self.config_path).exists())
    }
    
    async fn backup_config(&self) -> MceptionResult<String> {
        if !self.config_exists().await? {
            return Err(MceptionError::Storage(StorageError::NotFound(
                "Configuration file not found for backup".to_string()
            )));
        }
        
        let backup_path = self.backup_path();
        fs::copy(&self.config_path, &backup_path)
            .await
            .map_err(StorageError::from)?;
            
        Ok(backup_path)
    }
}
