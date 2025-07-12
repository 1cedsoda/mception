pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mception-server")]
#[command(about = "MCePtion Server - MCP hotplugging system for distributed agents")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "data/config.json")]
    pub config: String,

    /// Audit log file path
    #[arg(short, long, default_value = "data/audit.log")]
    pub audit_log: String,

    /// Server bind address
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCePtion server (default)
    Start,
    /// Show current configuration
    ShowConfig {
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Show audit log entries
    ShowAudit {
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
        /// Number of recent entries to show
        #[arg(short, long)]
        limit: Option<usize>,
        /// Filter by action type
        #[arg(long)]
        action: Option<String>,
        /// Filter by target type
        #[arg(long)]
        target: Option<String>,
        /// Filter by actor
        #[arg(long)]
        actor: Option<String>,
    },
    /// List all leaf MCPs
    ListMcps {
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// List all agents
    ListAgents {
        /// Output format
        #[arg(short, long, default_value = "pretty")]
        format: OutputFormat,
    },
    /// Export configuration to a file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: OutputFormat,
    },
}

#[derive(Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Pretty,
    Yaml,
    Table,
}

impl Default for Commands {
    fn default() -> Self {
        Commands::Start
    }
}
