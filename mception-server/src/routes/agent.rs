use axum::{
    Router,
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
    routing::{any, get},
};
use serde_json::Value;
use std::sync::Arc;

use crate::services::ConfigService;

type ServiceExtension = Extension<Arc<ConfigService>>;

pub fn router() -> Router {
    Router::new()
        .route("/:agent_id/config", get(get_agent_config))
        .route("/:agent_id/forwarding", any(agent_forwarding))
        .route("/:agent_id/forwarding_ws", any(agent_forwarding_ws))
}

async fn get_agent_config(
    Extension(service): ServiceExtension,
    Path(agent_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match service.get_agent_remote_config(&agent_id).await {
        Ok(config) => Ok(Json(config)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn agent_forwarding(
    Extension(_service): ServiceExtension,
    Path(_agent_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement MCP query forwarding to agents via WebSocket
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn agent_forwarding_ws(
    Extension(_service): ServiceExtension,
    Path(_agent_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement WebSocket connection for agent forwarding
    Err(StatusCode::NOT_IMPLEMENTED)
}
