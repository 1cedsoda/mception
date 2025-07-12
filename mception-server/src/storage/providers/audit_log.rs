use crate::core::{AuditLogEntry, MceptionResult};
use async_trait::async_trait;

/// Trait for audit log storage providers
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// Append a new audit log entry
    async fn append_entry(&self, entry: &AuditLogEntry) -> MceptionResult<()>;

    /// Load all audit log entries
    async fn load_entries(&self) -> MceptionResult<Vec<AuditLogEntry>>;
}
