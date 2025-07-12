use axum::{
    Router,
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
    routing::any,
};
use serde_json::Value;
use std::sync::Arc;

use crate::services::ConfigService;

type ServiceExtension = Extension<Arc<ConfigService>>;

pub fn router() -> Router {
    Router::new().route("/:leaf_mcp_id/forwarding", any(leaf_mcp_forwarding))
}

async fn leaf_mcp_forwarding(
    Extension(_service): ServiceExtension,
    Path(_leaf_mcp_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement MCP query forwarding to leaf MCPs
    // This should forward requests to the actual MCP server (STDIO or HTTPS)
    Err(StatusCode::NOT_IMPLEMENTED)
}
