use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Cypher CLI - Production-grade cybersecurity auditing CLI for developers
#[derive(Parser, Debug)]
#[command(name = "cypher")]
#[command(author = "Cypher Security Team")]
#[command(version = "0.1.0")]
#[command(about = "Static security analysis for your codebase", long_about = None)]
pub struct Cli {
    /// Increase verbosity level (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global =true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a directory or file for security issues
    Scan {
        /// Path to scan (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Output format (json, sarif, html, text)
        #[arg(short, long, default_value = "text")]
        output: String,

        /// Output file path
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Severity threshold (low, medium, high, critical)
        #[arg(short, long, default_value = "low")]
        severity: String,

        /// Specific rules to run (comma-separated)
        #[arg(short, long)]
        rules: Option<String>,

        /// Rules to exclude (comma-separated)
        #[arg(short, long)]
        exclude_rules: Option<String>,

        /// Maximum number of issues to report
        #[arg(short, long)]
        max_issues: Option<usize>,

        /// Exit with non-zero status if issues found
        #[arg(long)]
        fail_on_issues: bool,
    },

    /// Automatically suggest or apply fixes for security vulnerabilities
    Fix {
        /// Path to scan and fix (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Run without applying changes (preview mode)
        #[arg(long)]
        dry_run: bool,
    },

    /// Initialize Cypher configuration
    Init {
        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// List available security rules
    ListRules {
        /// Filter by severity
        #[arg(short, long)]
        severity: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },

    /// Validate configuration file
    Validate {
        /// Path to configuration file
        #[arg(value_name = "CONFIG")]
        config: PathBuf,
    },

    /// Generate a security report
    Report {
        /// Path to scan
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Report format (json, sarif, html, pdf)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },

    /// Ask a security question to the Cypher AI Assistant
    Ask {
        /// The security question or prompt (optional; starts interactive chat if omitted)
        #[arg(value_name = "PROMPT")]
        prompt: Option<String>,

        /// API Key for the AI provider (falls back to provider-specific environment variables or keyring)
        #[arg(short, long)]
        api_key: Option<String>,
    },
}


#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List installed plugins
    List,

    /// Install a plugin
    Install {
        /// Plugin name or URL
        #[arg(value_name = "PLUGIN")]
        plugin: String,
    },

    /// Remove a plugin
    Remove {
        /// Plugin name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Update all plugins
    Update,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
