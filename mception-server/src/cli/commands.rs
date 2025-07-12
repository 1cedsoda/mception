use crate::{
    cli::{Commands, OutputFormat},
    core::{AuditLogEntry, AuditTarget, ServerConfig},
    services::ConfigService,
    storage::providers::{AuditStorage, ConfigStorage},
};
use serde_json;

pub async fn handle_command(
    command: Commands,
    manager: &ConfigService,
    config_storage: &dyn ConfigStorage,
    audit_storage: &dyn AuditStorage,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Start => {
            // This is handled in main.rs - just return Ok for now
            Ok(())
        }
        Commands::ShowConfig { format } => {
            let config = config_storage.load_config().await?;
            display_config(&config, format).await
        }
        Commands::ShowAudit {
            format,
            limit,
            action,
            target,
            actor,
        } => {
            let entries = audit_storage.load_entries().await?;
            let filtered_entries = filter_audit_entries(entries, limit, action, target, actor);
            display_audit_entries(&filtered_entries, format).await
        }
        Commands::ListMcps { format } => {
            let mcps = manager.list_leaf_mcps().await?;
            display_mcps_list(&mcps, format).await
        }
        Commands::ListAgents { format } => {
            let agents = manager.list_agents().await?;
            display_agents_list(&agents, format).await
        }
        Commands::Export { output, format } => {
            let config = config_storage.load_config().await?;
            export_config(&config, &output, format).await
        }
    }
}

async fn display_config(
    config: &ServerConfig,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(config)?);
        }
        OutputFormat::Pretty => {
            println!("MCePtion Server Configuration");
            println!("============================");
            println!("Version: {}", config.metadata.version);
            println!("Created: {}", config.metadata.created_at);
            println!("Last Modified: {}", config.metadata.last_modified);
            println!();

            println!("Leaf MCPs ({}):", config.leaf_mcps.len());
            for (id, mcp) in &config.leaf_mcps {
                println!("  - {}: {}", id, mcp.name.as_deref().unwrap_or("(no name)"));
                println!("    Transport: {:?}", mcp.transport);
                println!(
                    "    Local: {}, Reachable: {}",
                    mcp.is_local, mcp.reachable_by_agent
                );
            }
            println!();

            println!("MCePtion Agents ({}):", config.agents.len());
            for (id, agent) in &config.agents {
                println!(
                    "  - {}: {}",
                    id,
                    agent.name.as_deref().unwrap_or("(no name)")
                );
                println!("    Connected: {}", agent.is_connected);
                println!("    Allowed MCPs: {:?}", agent.allowed_mcp_ids);
                if let Some(last_seen) = agent.last_seen {
                    println!("    Last Seen: {}", last_seen);
                }
            }
        }
        OutputFormat::Yaml => {
            // For now, use JSON format as yaml crate would need to be added
            println!("# YAML output not implemented, showing JSON:");
            println!("{}", serde_json::to_string_pretty(config)?);
        }
        OutputFormat::Table => {
            println!("MCePtion Server Configuration Summary");
            println!("=====================================");
            println!("| Component      | Count | Details");
            println!("| -------------- | ----- | -------");
            println!(
                "| Leaf MCPs      | {:>5} | {}",
                config.leaf_mcps.len(),
                config
                    .leaf_mcps
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "| Agents         | {:>5} | {}",
                config.agents.len(),
                config.agents.keys().cloned().collect::<Vec<_>>().join(", ")
            );
            println!("| Version        |       | {}", config.metadata.version);
            println!(
                "| Last Modified  |       | {}",
                config.metadata.last_modified
            );
        }
    }
    Ok(())
}

async fn display_audit_entries(
    entries: &[AuditLogEntry],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(entries)?);
        }
        OutputFormat::Pretty => {
            println!("Audit Log Entries ({}):", entries.len());
            println!("======================");
            for entry in entries {
                println!("ID: {}", entry.id);
                println!("Timestamp: {}", entry.timestamp);
                println!("Action: {:?}", entry.action);
                println!("Target: {:?}", entry.target);
                if let Some(actor) = &entry.actor {
                    println!("Actor: {}", actor);
                }
                if let Some(reason) = &entry.reason {
                    println!("Reason: {}", reason);
                }
                if !entry.details.is_null() {
                    println!("Details: {}", serde_json::to_string_pretty(&entry.details)?);
                }
                println!("---");
            }
        }
        OutputFormat::Yaml => {
            println!("# YAML output not implemented, showing JSON:");
            println!("{}", serde_json::to_string_pretty(entries)?);
        }
        OutputFormat::Table => {
            println!("| Timestamp           | Action | Target Type | Target ID | Actor | Reason");
            println!("| ------------------- | ------ | ----------- | --------- | ----- | ------");
            for entry in entries {
                let target_info = match &entry.target {
                    AuditTarget::LeafMcp { id } => ("LeafMcp", id.as_str()),
                    AuditTarget::Agent { id } => ("Agent", id.as_str()),
                    AuditTarget::AgentAllowedMcp {
                        agent_id,
                        mcp_id: _,
                    } => ("AgentMcp", agent_id.as_str()),
                    AuditTarget::Server => ("Server", ""),
                };
                println!(
                    "| {} | {:?} | {} | {} | {} | {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.action,
                    target_info.0,
                    target_info.1,
                    entry.actor.as_deref().unwrap_or(""),
                    entry.reason.as_deref().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}

async fn display_mcps_list(
    mcps: &[(String, crate::core::LeafMcpConfig)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(mcps)?);
        }
        OutputFormat::Pretty => {
            println!("Leaf MCPs ({}):", mcps.len());
            for (id, config) in mcps {
                let name = config.name.as_ref().unwrap_or(id);
                println!("  - {} ({})", id, name);
                if let Some(desc) = &config.description {
                    println!("    Description: {}", desc);
                }
            }
        }
        OutputFormat::Yaml => {
            println!("# YAML output not implemented, showing JSON:");
            println!("{}", serde_json::to_string_pretty(mcps)?);
        }
        OutputFormat::Table => {
            println!("| ID | Name | Description |");
            println!("| -- | ---- | ----------- |");
            for (id, config) in mcps {
                let name = config.name.as_ref().unwrap_or(id);
                let desc = config
                    .description
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("");
                println!("| {} | {} | {} |", id, name, desc);
            }
        }
    }
    Ok(())
}

async fn display_agents_list(
    agents: &[(String, crate::core::AgentConfig)],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(agents)?);
        }
        OutputFormat::Pretty => {
            println!("MCePtion Agents ({}):", agents.len());
            for (id, config) in agents {
                let name = config.name.as_ref().unwrap_or(id);
                println!("  - {} ({})", id, name);
                println!("    Connected: {}", config.is_connected);
                println!("    Allowed MCPs: {}", config.allowed_mcp_ids.len());
                if let Some(desc) = &config.description {
                    println!("    Description: {}", desc);
                }
            }
        }
        OutputFormat::Yaml => {
            println!("# YAML output not implemented, showing JSON:");
            println!("{}", serde_json::to_string_pretty(agents)?);
        }
        OutputFormat::Table => {
            println!("| Agent ID | Name | Connected | Allowed MCPs |");
            println!("| -------- | ---- | --------- | ------------ |");
            for (id, config) in agents {
                let name = config.name.as_ref().unwrap_or(id);
                println!(
                    "| {} | {} | {} | {} |",
                    id,
                    name,
                    config.is_connected,
                    config.allowed_mcp_ids.len()
                );
            }
        }
    }
    Ok(())
}

async fn export_config(
    config: &ServerConfig,
    output: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;

    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(config)?,
        OutputFormat::Pretty => {
            format!(
                "MCePtion Server Configuration Export\n\
                     Generated: {}\n\
                     {}",
                chrono::Utc::now(),
                serde_json::to_string_pretty(config)?
            )
        }
        OutputFormat::Yaml => {
            format!(
                "# MCePtion Server Configuration Export\n\
                     # Generated: {}\n\
                     # YAML export not fully implemented, using JSON format:\n\
                     {}",
                chrono::Utc::now(),
                serde_json::to_string_pretty(config)?
            )
        }
        OutputFormat::Table => {
            format!(
                "MCePtion Server Configuration Export (Table Format)\n\
                     Generated: {}\n\
                     Note: Table format is not suitable for export, using JSON:\n\
                     {}",
                chrono::Utc::now(),
                serde_json::to_string_pretty(config)?
            )
        }
    };

    // Create directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(output).parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(output, content).await?;
    println!("Configuration exported to: {}", output);
    Ok(())
}

fn filter_audit_entries(
    entries: Vec<AuditLogEntry>,
    limit: Option<usize>,
    action_filter: Option<String>,
    target_filter: Option<String>,
    actor_filter: Option<String>,
) -> Vec<AuditLogEntry> {
    let mut filtered: Vec<AuditLogEntry> = entries
        .into_iter()
        .filter(|entry| {
            // Filter by action
            if let Some(action) = &action_filter {
                let action_str = format!("{:?}", entry.action).to_lowercase();
                if !action_str.contains(&action.to_lowercase()) {
                    return false;
                }
            }

            // Filter by target type
            if let Some(target) = &target_filter {
                let target_str = match &entry.target {
                    AuditTarget::LeafMcp { .. } => "leafmcp",
                    AuditTarget::Agent { .. } => "agent",
                    AuditTarget::AgentAllowedMcp { .. } => "agentallowedmcp",
                    AuditTarget::Server => "server",
                };
                if !target_str.contains(&target.to_lowercase()) {
                    return false;
                }
            }

            // Filter by actor
            if let Some(actor) = &actor_filter {
                if let Some(entry_actor) = &entry.actor {
                    if !entry_actor.to_lowercase().contains(&actor.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .collect();

    // Sort by timestamp (newest first)
    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    if let Some(limit) = limit {
        filtered.truncate(limit);
    }

    filtered
}
