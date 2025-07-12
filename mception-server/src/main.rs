mod cli;
mod core;
mod routes;
mod services;
mod storage;

use axum::{Extension, Router};
use clap::Parser;
use cli::{Cli, Commands};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

use crate::services::ConfigService;
use crate::storage::providers::{FileAuditStorage, FileConfigStorage};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Initialize storage providers with CLI-provided paths
    let config_storage = Arc::new(FileConfigStorage::new(&cli.config));
    let audit_storage = Arc::new(FileAuditStorage::new(&cli.audit_log));
    let config_service = Arc::new(ConfigService::new(
        config_storage.clone(),
        audit_storage.clone(),
    ));

    // Load existing configuration
    if let Err(e) = config_service.load_configuration().await {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }

    // Handle CLI commands
    match cli.command.unwrap_or_default() {
        Commands::Start => {
            // Start the server
            start_server(config_service, cli.host, cli.port).await;
        }
        _command => {
            // Handle other commands
            if let Err(e) = cli::commands::handle_command(
                _command,
                &*config_service,
                config_storage.as_ref(),
                audit_storage.as_ref(),
            )
            .await
            {
                error!("Error executing command: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn start_server(config_service: Arc<ConfigService>, host: String, port: u16) {
    let app = Router::new()
        // Admin API routes (no /admin prefix per README spec)
        .nest("/admin", routes::admin::router())
        // Agent runtime routes (with /agent prefix)
        .nest("/agent", routes::agent::router())
        // Leaf MCP forwarding routes (with /leaf prefix)
        .nest("/leaf", routes::leaf::router())
        .layer(Extension(config_service.clone()));

    let addr = SocketAddr::from((
        host.parse::<std::net::IpAddr>()
            .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0))),
        port,
    ));

    info!("MCePtion Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
