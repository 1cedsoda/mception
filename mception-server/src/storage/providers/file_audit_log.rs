use super::audit_log::AuditStorage;
use crate::core::{AuditLogEntry, MceptionResult, StorageError};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

/// File-based audit log storage implementation
#[derive(Debug, Clone)]
pub struct FileAuditStorage {
    audit_log_path: String,
}

impl FileAuditStorage {
    pub fn new(audit_log_path: impl Into<String>) -> Self {
        Self {
            audit_log_path: audit_log_path.into(),
        }
    }
    
    /// Initialize the audit log file if it doesn't exist
    pub async fn initialize(&self) -> MceptionResult<()> {
        if !Path::new(&self.audit_log_path).exists() {
            // Create directory if it doesn't exist
            if let Some(parent) = Path::new(&self.audit_log_path).parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(StorageError::from)?;
            }
            
            // Create an empty audit log file
            fs::write(&self.audit_log_path, "")
                .await
                .map_err(StorageError::from)?;
        }
        Ok(())
    }
}

#[async_trait]
impl AuditStorage for FileAuditStorage {
    async fn append_entry(&self, entry: &AuditLogEntry) -> MceptionResult<()> {
        let content = serde_json::to_string(entry).map_err(StorageError::from)? + "\n";

        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&self.audit_log_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(StorageError::from)?;
        }

        // Append to file
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)
            .await
            .map_err(StorageError::from)?;

        file.write_all(content.as_bytes())
            .await
            .map_err(StorageError::from)?;
        file.flush().await.map_err(StorageError::from)?;

        Ok(())
    }

    async fn load_entries(&self) -> MceptionResult<Vec<AuditLogEntry>> {
        if !Path::new(&self.audit_log_path).exists() {
            // Initialize the audit log file
            self.initialize().await?;
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.audit_log_path)
            .await
            .map_err(StorageError::from)?;
            
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
            
        let mut logs = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                let entry: AuditLogEntry =
                    serde_json::from_str(line).map_err(StorageError::from)?;
                logs.push(entry);
            }
        }

        Ok(logs)
    }
}
