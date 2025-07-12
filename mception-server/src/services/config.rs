use crate::core::{
    AgentConfig, AuditAction, AuditLogEntry, AuditTarget, LeafMcpConfig, MceptionError,
    MceptionResult, ServerConfig, StorageError, ValidationError,
};
use crate::storage::providers::{AuditStorage, ConfigStorage};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// The main service for managing MCeption server configuration and operations
pub struct ConfigService {
    config: Arc<RwLock<ServerConfig>>,
    config_storage: Arc<dyn ConfigStorage>,
    audit_storage: Arc<dyn AuditStorage>,
}

impl ConfigService {
    pub fn new(
        config_storage: Arc<dyn ConfigStorage>,
        audit_storage: Arc<dyn AuditStorage>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(ServerConfig::default())),
            config_storage,
            audit_storage,
        }
    }

    /// Load configuration from storage
    pub async fn load_configuration(&self) -> MceptionResult<()> {
        let config = self.config_storage.load_config().await?;
        *self.config.write().await = config;
        Ok(())
    }

    /// Save current configuration to storage
    pub async fn save_configuration(&self) -> MceptionResult<()> {
        let config = self.config.read().await;
        self.config_storage.save_config(&*config).await?;
        Ok(())
    }

    /// Get a read-only copy of the current server configuration
    pub async fn get_configuration(&self) -> ServerConfig {
        self.config.read().await.clone()
    }

    /// Create a backup of the current configuration
    pub async fn backup_configuration(&self) -> MceptionResult<String> {
        self.config_storage.backup_config().await
    }

    /// Log an audit entry
    async fn audit_log(
        &self,
        action: AuditAction,
        target: AuditTarget,
        actor: Option<String>,
        reason: Option<String>,
        details: serde_json::Value,
    ) -> MceptionResult<()> {
        let entry = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action,
            actor,
            target,
            reason,
            details,
        };

        self.audit_storage.append_entry(&entry).await?;
        Ok(())
    }

    // Leaf MCP operations

    /// Create a new leaf MCP configuration
    pub async fn create_leaf_mcp(
        &self,
        id: String,
        config: LeafMcpConfig,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        // Validation
        if id.trim().is_empty() {
            return Err(MceptionError::Validation(ValidationError::InvalidFormat(
                "MCP ID cannot be empty".to_string(),
            )));
        }

        let mut server_config = self.config.write().await;

        if server_config.leaf_mcps.contains_key(&id) {
            return Err(MceptionError::Storage(StorageError::AlreadyExists(
                format!("Leaf MCP with ID '{}' already exists", id),
            )));
        }

        server_config.leaf_mcps.insert(id.clone(), config.clone());
        server_config.update_last_modified();

        // Release the lock before async operations
        drop(server_config);

        self.audit_log(
            AuditAction::Create,
            AuditTarget::LeafMcp { id: id.clone() },
            actor,
            reason,
            serde_json::to_value(&config).unwrap_or_default(),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Read a leaf MCP configuration
    pub async fn get_leaf_mcp(
        &self,
        id: &str,
        actor: Option<String>,
    ) -> MceptionResult<LeafMcpConfig> {
        let config = self.config.read().await;
        let mcp_config = config
            .leaf_mcps
            .get(id)
            .ok_or_else(|| {
                MceptionError::Storage(StorageError::NotFound(format!(
                    "Leaf MCP with ID '{}' not found",
                    id
                )))
            })?
            .clone();

        drop(config);

        // Log read access (but don't fail if audit logging fails)
        if let Err(e) = self
            .audit_log(
                AuditAction::Read,
                AuditTarget::LeafMcp { id: id.to_string() },
                actor,
                None,
                serde_json::Value::Null,
            )
            .await
        {
            // Log the error but don't propagate it for read operations
            eprintln!("Failed to log audit entry for read operation: {}", e);
        }

        Ok(mcp_config)
    }

    /// List all leaf MCP configurations
    pub async fn list_leaf_mcps(&self) -> MceptionResult<Vec<(String, LeafMcpConfig)>> {
        let config = self.config.read().await;
        let mcps = config
            .leaf_mcps
            .iter()
            .map(|(id, config)| (id.clone(), config.clone()))
            .collect();
        Ok(mcps)
    }

    /// Update a leaf MCP configuration
    pub async fn update_leaf_mcp(
        &self,
        id: &str,
        updates: serde_json::Value,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        let mcp_config = server_config.leaf_mcps.get_mut(id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Leaf MCP with ID '{}' not found",
                id
            )))
        })?;

        // Apply partial updates
        if let serde_json::Value::Object(ref updates_map) = updates {
            let config_value = serde_json::to_value(&*mcp_config).map_err(|e| {
                MceptionError::Validation(ValidationError::InvalidFormat(e.to_string()))
            })?;

            if let serde_json::Value::Object(mut config_map) = config_value {
                for (key, value) in updates_map {
                    config_map.insert(key.clone(), value.clone());
                }
                *mcp_config = serde_json::from_value(serde_json::Value::Object(config_map))
                    .map_err(|e| {
                        MceptionError::Validation(ValidationError::InvalidFormat(e.to_string()))
                    })?;
            }
        }

        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::Update,
            AuditTarget::LeafMcp { id: id.to_string() },
            actor,
            reason,
            updates,
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Delete a leaf MCP configuration
    pub async fn delete_leaf_mcp(
        &self,
        id: &str,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        let removed_config = server_config.leaf_mcps.remove(id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Leaf MCP with ID '{}' not found",
                id
            )))
        })?;

        // Remove from all agents' allowed_mcp_ids
        for agent in server_config.agents.values_mut() {
            agent.allowed_mcp_ids.retain(|mcp_id| mcp_id != id);
        }

        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::Delete,
            AuditTarget::LeafMcp { id: id.to_string() },
            actor,
            reason,
            serde_json::to_value(&removed_config).unwrap_or_default(),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    // Agent operations

    /// Create a new agent configuration
    pub async fn create_agent(
        &self,
        agent_id: String,
        allowed_mcp_ids: Vec<String>,
        actor: Option<String>,
    ) -> MceptionResult<()> {
        // Validation
        if agent_id.trim().is_empty() {
            return Err(MceptionError::Validation(ValidationError::InvalidFormat(
                "Agent ID cannot be empty".to_string(),
            )));
        }

        let mut server_config = self.config.write().await;

        if server_config.agents.contains_key(&agent_id) {
            return Err(MceptionError::Storage(StorageError::AlreadyExists(
                format!("Agent with ID '{}' already exists", agent_id),
            )));
        }

        // Validate that all allowed MCPs exist
        for mcp_id in &allowed_mcp_ids {
            if !server_config.leaf_mcps.contains_key(mcp_id)
                && !server_config.agents.contains_key(mcp_id)
            {
                return Err(MceptionError::Validation(ValidationError::InvalidFormat(
                    format!("MCP with ID '{}' does not exist", mcp_id),
                )));
            }
        }

        let agent_config = AgentConfig {
            agent_id: agent_id.clone(),
            name: None,
            description: None,
            allowed_mcp_ids: allowed_mcp_ids.clone(),
            is_connected: false,
            last_seen: None,
            config: serde_json::Value::Object(serde_json::Map::new()),
        };

        server_config
            .agents
            .insert(agent_id.clone(), agent_config.clone());
        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::Create,
            AuditTarget::Agent {
                id: agent_id.clone(),
            },
            actor,
            None,
            serde_json::to_value(&agent_config).unwrap_or_default(),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Get an agent configuration
    pub async fn get_agent(
        &self,
        agent_id: &str,
        actor: Option<String>,
    ) -> MceptionResult<AgentConfig> {
        let config = self.config.read().await;
        let agent_config = config
            .agents
            .get(agent_id)
            .ok_or_else(|| {
                MceptionError::Storage(StorageError::NotFound(format!(
                    "Agent with ID '{}' not found",
                    agent_id
                )))
            })?
            .clone();

        drop(config);

        // Log read access (but don't fail if audit logging fails)
        if let Err(e) = self
            .audit_log(
                AuditAction::Read,
                AuditTarget::Agent {
                    id: agent_id.to_string(),
                },
                actor,
                None,
                serde_json::Value::Null,
            )
            .await
        {
            eprintln!("Failed to log audit entry for read operation: {}", e);
        }

        Ok(agent_config)
    }

    /// List all agent configurations
    pub async fn list_agents(&self) -> MceptionResult<Vec<(String, AgentConfig)>> {
        let config = self.config.read().await;
        let agents = config
            .agents
            .iter()
            .map(|(id, config)| (id.clone(), config.clone()))
            .collect();
        Ok(agents)
    }

    /// Update an agent configuration
    pub async fn update_agent(
        &self,
        agent_id: &str,
        updates: serde_json::Value,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        let agent_config = server_config.agents.get_mut(agent_id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Agent with ID '{}' not found",
                agent_id
            )))
        })?;

        // Apply partial updates
        if let serde_json::Value::Object(ref updates_map) = updates {
            let config_value = serde_json::to_value(&*agent_config).map_err(|e| {
                MceptionError::Validation(ValidationError::InvalidFormat(e.to_string()))
            })?;

            if let serde_json::Value::Object(mut config_map) = config_value {
                for (key, value) in updates_map {
                    config_map.insert(key.clone(), value.clone());
                }
                *agent_config = serde_json::from_value(serde_json::Value::Object(config_map))
                    .map_err(|e| {
                        MceptionError::Validation(ValidationError::InvalidFormat(e.to_string()))
                    })?;
            }
        }

        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::Update,
            AuditTarget::Agent {
                id: agent_id.to_string(),
            },
            actor,
            reason,
            updates,
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Delete an agent configuration
    pub async fn delete_agent(
        &self,
        agent_id: &str,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        let removed_config = server_config.agents.remove(agent_id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Agent with ID '{}' not found",
                agent_id
            )))
        })?;

        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::Delete,
            AuditTarget::Agent {
                id: agent_id.to_string(),
            },
            actor,
            reason,
            serde_json::to_value(&removed_config).unwrap_or_default(),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Add an allowed MCP to an agent
    pub async fn add_agent_allowed_mcp(
        &self,
        agent_id: &str,
        mcp_id: &str,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        // Check if MCP exists
        if !server_config.leaf_mcps.contains_key(mcp_id)
            && !server_config.agents.contains_key(mcp_id)
        {
            return Err(MceptionError::Validation(ValidationError::InvalidFormat(
                format!("MCP with ID '{}' does not exist", mcp_id),
            )));
        }

        let agent_config = server_config.agents.get_mut(agent_id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Agent with ID '{}' not found",
                agent_id
            )))
        })?;

        // Check if MCP is already allowed
        if agent_config.allowed_mcp_ids.contains(&mcp_id.to_string()) {
            return Err(MceptionError::Storage(StorageError::AlreadyExists(
                format!(
                    "MCP '{}' is already allowed for agent '{}'",
                    mcp_id, agent_id
                ),
            )));
        }

        agent_config.allowed_mcp_ids.push(mcp_id.to_string());
        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::AddAllowedMcp,
            AuditTarget::AgentAllowedMcp {
                agent_id: agent_id.to_string(),
                mcp_id: mcp_id.to_string(),
            },
            actor,
            reason,
            serde_json::json!({ "mcp_id": mcp_id }),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Remove an allowed MCP from an agent
    pub async fn remove_agent_allowed_mcp(
        &self,
        agent_id: &str,
        mcp_id: &str,
        actor: Option<String>,
        reason: Option<String>,
    ) -> MceptionResult<()> {
        let mut server_config = self.config.write().await;

        let agent_config = server_config.agents.get_mut(agent_id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Agent with ID '{}' not found",
                agent_id
            )))
        })?;

        // Check if MCP is currently allowed
        if !agent_config.allowed_mcp_ids.contains(&mcp_id.to_string()) {
            return Err(MceptionError::Storage(StorageError::NotFound(format!(
                "MCP '{}' is not allowed for agent '{}'",
                mcp_id, agent_id
            ))));
        }

        agent_config.allowed_mcp_ids.retain(|id| id != mcp_id);
        server_config.update_last_modified();
        drop(server_config);

        self.audit_log(
            AuditAction::RemoveAllowedMcp,
            AuditTarget::AgentAllowedMcp {
                agent_id: agent_id.to_string(),
                mcp_id: mcp_id.to_string(),
            },
            actor,
            reason,
            serde_json::json!({ "mcp_id": mcp_id }),
        )
        .await?;

        self.save_configuration().await?;
        Ok(())
    }

    /// Get audit log entries
    pub async fn get_audit_logs(&self) -> MceptionResult<Vec<AuditLogEntry>> {
        self.audit_storage.load_entries().await
    }

    /// Get the remote configuration for an agent (filtered MCPs that the agent is allowed to use)
    pub async fn get_agent_remote_config(
        &self,
        agent_id: &str,
    ) -> MceptionResult<serde_json::Value> {
        let config = self.config.read().await;

        let agent = config.agents.get(agent_id).ok_or_else(|| {
            MceptionError::Storage(StorageError::NotFound(format!(
                "Agent with ID '{}' not found",
                agent_id
            )))
        })?;

        // Build the remote config with only allowed MCPs
        let mut remote_mcps = serde_json::Map::new();

        for mcp_id in &agent.allowed_mcp_ids {
            if let Some(mcp_config) = config.leaf_mcps.get(mcp_id) {
                remote_mcps.insert(
                    mcp_id.clone(),
                    serde_json::to_value(mcp_config).unwrap_or_default(),
                );
            } else if let Some(agent_config) = config.agents.get(mcp_id) {
                // Include other agents that this agent can use
                remote_mcps.insert(
                    mcp_id.clone(),
                    serde_json::to_value(agent_config).unwrap_or_default(),
                );
            }
        }

        let remote_config = serde_json::json!({
            "agent_id": agent_id,
            "mcps": remote_mcps,
            "metadata": {
                "last_updated": config.metadata.last_modified,
                "version": config.metadata.version
            }
        });

        Ok(remote_config)
    }
}
