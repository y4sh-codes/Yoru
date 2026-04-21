//! Command-line argument definitions.
//!
//! Doctag:cli-args

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Main Yoru CLI args.
#[derive(Debug, Parser)]
#[command(
    name = "yoru",
    version,
    about = "A lightweight Postman-like API client for terminal workflows"
)]
pub struct Cli {
    /// Optional custom data directory.
    #[arg(long, env = "YORU_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Supported top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch interactive terminal UI.
    Tui,

    /// Execute a single request directly from CLI.
    Send(SendArgs),

    /// Initialize workspace storage with starter content.
    Init {
        #[arg(long)]
        name: Option<String>,
    },

    /// Import workspace from JSON or YAML.
    Import {
        #[arg(long)]
        file: PathBuf,
    },

    /// Export current workspace to JSON or YAML.
    Export {
        #[arg(long)]
        file: PathBuf,
    },
}

/// Arguments for one-off request execution.
#[derive(Debug, Parser)]
pub struct SendArgs {
    #[arg(long, default_value = "GET")]
    pub method: String,

    #[arg(long)]
    pub url: String,

    #[arg(long)]
    pub name: Option<String>,

    /// Header entries in `Key:Value` format.
    #[arg(long = "header", short = 'H')]
    pub headers: Vec<String>,

    /// Query entries in `key=value` format.
    #[arg(long = "query", short = 'q')]
    pub query: Vec<String>,

    /// Raw request body text.
    #[arg(long)]
    pub data: Option<String>,

    /// JSON body value.
    #[arg(long)]
    pub json: Option<String>,

    /// Bearer token auth.
    #[arg(long)]
    pub bearer: Option<String>,

    /// Basic auth username.
    #[arg(long)]
    pub basic_user: Option<String>,

    /// Basic auth password.
    #[arg(long)]
    pub basic_password: Option<String>,

    /// API key pair in `key=value` format.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Place API key in query string (default places in header).
    #[arg(long)]
    pub api_key_in_query: bool,

    /// Environment variables in `key=value` format.
    #[arg(long = "env", short = 'e')]
    pub env: Vec<String>,

    /// Request timeout in milliseconds.
    #[arg(long)]
    pub timeout_ms: Option<u64>,

    /// Optional inline pre-request script.
    #[arg(long)]
    pub pre_script: Option<String>,

    /// Optional inline test script.
    #[arg(long)]
    pub test_script: Option<String>,
}
