use axum::{
    Router,
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde_json::Value;
use std::sync::Arc;

use crate::core::{
    AddAgentAllowedMcpRequest, CreateAgentRequest, CreateLeafMcpRequest, DeleteAgentRequest,
    DeleteLeafMcpRequest, LeafMcpConfig, RemoveAgentAllowedMcpRequest, UpdateAgentRequest,
    UpdateLeafMcpRequest,
};
use crate::services::ConfigService;

type ServiceExtension = Extension<Arc<ConfigService>>;

pub fn router() -> Router {
    Router::new()
        // Leaf MCP endpoints
        .route("/leaf", post(create_leaf_mcp))
        .route("/leaf/:leaf_mcp_id/config", get(read_leaf_mcp_config))
        .route("/leaf/:leaf_mcp_id/config", put(update_leaf_mcp_config))
        .route("/leaf/:leaf_mcp_id", delete(delete_leaf_mcp))
        .route("/leaf/:leaf_mcp_id/tools", get(read_leaf_mcp_tools))
        // MCeption Agent endpoints
        .route("/agent", post(create_agent))
        .route("/agent/:agent_id/config", get(read_agent_config))
        .route("/agent/:agent_id/config", put(update_agent_config))
        .route("/agent/:agent_id", delete(delete_agent))
        .route("/agent/:agent_id/tools", get(read_agent_tools))
        .route(
            "/agent/:agent_id/allowed_mcps",
            post(add_agent_allowed_mcps),
        )
        .route(
            "/agent/:agent_id/allowed_mcps",
            delete(remove_agent_allowed_mcps),
        )
        // System endpoints
        .route("/config", get(get_server_config))
        .route("/config/backup", post(backup_server_config))
        .route("/audit", get(get_audit_logs))
}

// Leaf MCP handlers
async fn create_leaf_mcp(
    Extension(service): ServiceExtension,
    Json(request): Json<CreateLeafMcpRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_create {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .create_leaf_mcp(
            request.id.clone(),
            request.config,
            Some("admin".to_string()),
            request.reason,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Leaf MCP '{}' created successfully", request.id)
        }))),
        Err(e) => {
            eprintln!("Error creating leaf MCP: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn read_leaf_mcp_config(
    Extension(service): ServiceExtension,
    Path(leaf_mcp_id): Path<String>,
) -> Result<Json<LeafMcpConfig>, StatusCode> {
    match service
        .get_leaf_mcp(&leaf_mcp_id, Some("admin".to_string()))
        .await
    {
        Ok(config) => Ok(Json(config)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_leaf_mcp_config(
    Extension(service): ServiceExtension,
    Path(leaf_mcp_id): Path<String>,
    Json(request): Json<UpdateLeafMcpRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_update {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .update_leaf_mcp(
            &leaf_mcp_id,
            request.config,
            Some("admin".to_string()),
            request.reason,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Leaf MCP '{}' updated successfully", leaf_mcp_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_leaf_mcp(
    Extension(service): ServiceExtension,
    Path(leaf_mcp_id): Path<String>,
    Json(request): Json<DeleteLeafMcpRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_delete_mcp {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .delete_leaf_mcp(&leaf_mcp_id, Some("admin".to_string()), request.reason)
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Leaf MCP '{}' deleted successfully", leaf_mcp_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn read_leaf_mcp_tools(
    Extension(_service): ServiceExtension,
    Path(_leaf_mcp_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement actual MCP tool forwarding
    // For now, return empty tools list
    Ok(Json(serde_json::json!({
        "tools": []
    })))
}

// MCeption Agent handlers
async fn create_agent(
    Extension(service): ServiceExtension,
    Json(request): Json<CreateAgentRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_create {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .create_agent(
            request.agent_id.clone(),
            request.allowed_mcp_ids,
            Some("admin".to_string()),
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Agent '{}' created successfully", request.agent_id)
        }))),
        Err(e) => {
            eprintln!("Error creating agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn read_agent_config(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match service
        .get_agent(&agent_id, Some("admin".to_string()))
        .await
    {
        Ok(config) => Ok(Json(serde_json::json!({
            "allowed_mcp_ids": config.allowed_mcp_ids,
            "is_connected": config.is_connected,
            "last_seen": config.last_seen,
            "config": config.config
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_agent_config(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
    Json(request): Json<UpdateAgentRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_update {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .update_agent(
            &agent_id,
            request.config,
            Some("admin".to_string()),
            request.reason,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Agent '{}' updated successfully", agent_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_agent(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
    Json(request): Json<DeleteAgentRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_delete_mcp {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .delete_agent(&agent_id, Some("admin".to_string()), request.reason)
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Agent '{}' deleted successfully", agent_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn read_agent_tools(
    Extension(_service): ServiceExtension,
    Path(_agent_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement actual agent tool forwarding
    // For now, return empty tools list
    Ok(Json(serde_json::json!({
        "tools": []
    })))
}

async fn add_agent_allowed_mcps(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
    Json(request): Json<AddAgentAllowedMcpRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_add_mcp_id {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .add_agent_allowed_mcp(
            &agent_id,
            &request.mcp_id,
            Some("admin".to_string()),
            request.reason,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("MCP '{}' added to agent '{}' allowed list", request.mcp_id, agent_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn remove_agent_allowed_mcps(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
    Json(request): Json<RemoveAgentAllowedMcpRequest>,
) -> Result<Json<Value>, StatusCode> {
    if !request.should_remove_mcp_id {
        return Err(StatusCode::BAD_REQUEST);
    }

    match service
        .remove_agent_allowed_mcp(
            &agent_id,
            &request.mcp_id,
            Some("admin".to_string()),
            request.reason,
        )
        .await
    {
        Ok(()) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("MCP '{}' removed from agent '{}' allowed list", request.mcp_id, agent_id)
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// System handlers
async fn get_server_config(
    Extension(service): ServiceExtension,
) -> Result<Json<Value>, StatusCode> {
    let config = service.get_configuration().await;
    Ok(Json(serde_json::to_value(&config).unwrap_or_default()))
}

async fn backup_server_config(
    Extension(service): ServiceExtension,
) -> Result<Json<Value>, StatusCode> {
    match service.backup_configuration().await {
        Ok(backup_path) => Ok(Json(serde_json::json!({
            "success": true,
            "backup_path": backup_path,
            "message": "Configuration backup created successfully"
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_audit_logs(Extension(service): ServiceExtension) -> Result<Json<Value>, StatusCode> {
    match service.get_audit_logs().await {
        Ok(logs) => Ok(Json(serde_json::to_value(&logs).unwrap_or_default())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
