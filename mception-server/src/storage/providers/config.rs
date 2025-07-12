use crate::core::{ServerConfig, MceptionResult};
use async_trait::async_trait;

/// Trait for configuration storage providers
#[async_trait]
pub trait ConfigStorage: Send + Sync {
    /// Load the server configuration from storage
    async fn load_config(&self) -> MceptionResult<ServerConfig>;
    
    /// Save the server configuration to storage
    async fn save_config(&self, config: &ServerConfig) -> MceptionResult<()>;
    
    /// Check if configuration exists in storage
    async fn config_exists(&self) -> MceptionResult<bool>;
    
    /// Create a backup of the current configuration
    async fn backup_config(&self) -> MceptionResult<String>;
}
