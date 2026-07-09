use thiserror::Error;

/// Cypher CLI error types
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CypherError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Rule error: {0}")]
    Rule(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Report error: {0}")]
    Report(String),

    #[error("Scanner error: {0}")]
    Scanner(String),

    #[error("AST parsing error: {0}")]
    Ast(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid severity level: {0}")]
    InvalidSeverity(String),

    #[error("Invalid output format: {0}")]
    InvalidOutputFormat(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, CypherError>;

