use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Configuration for a leaf MCP (Model Context Protocol) server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafMcpConfig {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub transport: McpTransport,
    /// If the leaf MCP is hosted on the Agent system, not the server system
    pub is_local: bool,
    /// Whether the MCP is reachable by agents directly
    pub reachable_by_agent: bool,
    /// Additional configuration specific to the MCP
    pub config: serde_json::Value,
}

/// Transport configuration for MCP connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        env: Option<HashMap<String, String>>,
    },
    Https {
        url: String,
        headers: Option<HashMap<String, String>>,
    },
}

/// Represents an MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

/// Configuration for a MCeption Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// List of MCP IDs that this agent is allowed to use
    pub allowed_mcp_ids: Vec<String>,
    /// Whether the agent is currently connected
    pub is_connected: bool,
    /// Last time the agent was seen
    pub last_seen: Option<DateTime<Utc>>,
    /// Additional configuration for the agent
    pub config: serde_json::Value,
}

/// Complete server configuration containing all MCPs and agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// All leaf MCP configurations
    pub leaf_mcps: HashMap<String, LeafMcpConfig>,
    /// All MCeption Agent configurations
    pub agents: HashMap<String, AgentConfig>,
    /// Server metadata
    pub metadata: ServerMetadata,
}

/// Metadata about the server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

/// An entry in the audit log tracking configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub actor: Option<String>, // Agent ID or "admin" or "system"
    pub target: AuditTarget,
    pub reason: Option<String>,
    pub details: serde_json::Value,
}

/// Types of actions that can be audited
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    AddAllowedMcp,
    RemoveAllowedMcp,
}

/// Targets that can be acted upon and audited
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditTarget {
    LeafMcp { id: String },
    Agent { id: String },
    AgentAllowedMcp { agent_id: String, mcp_id: String },
    Server,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            leaf_mcps: HashMap::new(),
            agents: HashMap::new(),
            metadata: ServerMetadata {
                version: "0.1.0".to_string(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
            },
        }
    }
}

impl ServerConfig {
    pub fn update_last_modified(&mut self) {
        self.metadata.last_modified = Utc::now();
    }
}

// Request/Response types for the API
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLeafMcpRequest {
    pub id: String,
    pub config: LeafMcpConfig,
    pub reason: Option<String>,
    pub should_create: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLeafMcpRequest {
    pub config: serde_json::Value, // Partial update
    pub reason: Option<String>,
    pub should_update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteLeafMcpRequest {
    pub reason: Option<String>,
    pub should_delete_mcp: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub agent_id: String,
    pub allowed_mcp_ids: Vec<String>,
    pub should_create: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub config: serde_json::Value, // Partial update
    pub reason: Option<String>,
    pub should_update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddAgentAllowedMcpRequest {
    pub mcp_id: String,
    pub reason: Option<String>,
    pub should_add_mcp_id: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveAgentAllowedMcpRequest {
    pub mcp_id: String,
    pub reason: Option<String>,
    pub should_remove_mcp_id: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAgentRequest {
    pub reason: Option<String>,
    pub should_delete_mcp: bool,
}

// WebSocket forwarding types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ForwardingMessage {
    Request {
        request_id: String,
        url_params: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    },
    Response {
        request_id: String,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Option<String>,
    },
}
