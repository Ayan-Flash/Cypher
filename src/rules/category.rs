use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Categories for security rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleCategory {
    /// SQL Injection vulnerabilities
    SqlInjection,
    /// Cross-site scripting
    Xss,
    /// Command injection
    CommandInjection,
    /// Path traversal
    PathTraversal,
    /// Insecure cryptography
    Cryptography,
    /// Authentication and authorization
    Auth,
    /// Sensitive data exposure
    DataExposure,
    /// Dependency vulnerabilities
    Dependency,
    /// Configuration issues
    Configuration,
    /// Error handling
    ErrorHandling,
    /// Logging and monitoring
    Logging,
    /// Input validation
    InputValidation,
    /// API security
    ApiSecurity,
    /// Hardcoded secrets
    HardcodedSecrets,
    /// Insecure deserialization
    Deserialization,
    /// Race conditions
    RaceCondition,
}

impl RuleCategory {
    /// Get all categories
    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            RuleCategory::SqlInjection,
            RuleCategory::Xss,
            RuleCategory::CommandInjection,
            RuleCategory::PathTraversal,
            RuleCategory::Cryptography,
            RuleCategory::Auth,
            RuleCategory::DataExposure,
            RuleCategory::Dependency,
            RuleCategory::Configuration,
            RuleCategory::ErrorHandling,
            RuleCategory::Logging,
            RuleCategory::InputValidation,
            RuleCategory::ApiSecurity,
            RuleCategory::HardcodedSecrets,
            RuleCategory::Deserialization,
            RuleCategory::RaceCondition,
        ]
    }

    /// Get category description
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        match self {
            RuleCategory::SqlInjection => "SQL injection vulnerabilities",
            RuleCategory::Xss => "Cross-site scripting vulnerabilities",
            RuleCategory::CommandInjection => "Command injection vulnerabilities",
            RuleCategory::PathTraversal => "Path traversal vulnerabilities",
            RuleCategory::Cryptography => "Weak or insecure cryptography",
            RuleCategory::Auth => "Authentication and authorization issues",
            RuleCategory::DataExposure => "Sensitive data exposure",
            RuleCategory::Dependency => "Vulnerable dependencies",
            RuleCategory::Configuration => "Security misconfigurations",
            RuleCategory::ErrorHandling => "Insecure error handling",
            RuleCategory::Logging => "Insufficient logging or monitoring",
            RuleCategory::InputValidation => "Input validation issues",
            RuleCategory::ApiSecurity => "API security issues",
            RuleCategory::HardcodedSecrets => "Hardcoded secrets or credentials",
            RuleCategory::Deserialization => "Insecure deserialization",
            RuleCategory::RaceCondition => "Race condition vulnerabilities",
        }
    }
}

impl fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleCategory::SqlInjection => write!(f, "sql_injection"),
            RuleCategory::Xss => write!(f, "xss"),
            RuleCategory::CommandInjection => write!(f, "command_injection"),
            RuleCategory::PathTraversal => write!(f, "path_traversal"),
            RuleCategory::Cryptography => write!(f, "cryptography"),
            RuleCategory::Auth => write!(f, "auth"),
            RuleCategory::DataExposure => write!(f, "data_exposure"),
            RuleCategory::Dependency => write!(f, "dependency"),
            RuleCategory::Configuration => write!(f, "configuration"),
            RuleCategory::ErrorHandling => write!(f, "error_handling"),
            RuleCategory::Logging => write!(f, "logging"),
            RuleCategory::InputValidation => write!(f, "input_validation"),
            RuleCategory::ApiSecurity => write!(f, "api_security"),
            RuleCategory::HardcodedSecrets => write!(f, "hardcoded_secrets"),
            RuleCategory::Deserialization => write!(f, "deserialization"),
            RuleCategory::RaceCondition => write!(f, "race_condition"),
        }
    }
}

impl FromStr for RuleCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sql_injection" => Ok(RuleCategory::SqlInjection),
            "xss" => Ok(RuleCategory::Xss),
            "command_injection" => Ok(RuleCategory::CommandInjection),
            "path_traversal" => Ok(RuleCategory::PathTraversal),
            "cryptography" => Ok(RuleCategory::Cryptography),
            "auth" => Ok(RuleCategory::Auth),
            "data_exposure" => Ok(RuleCategory::DataExposure),
            "dependency" => Ok(RuleCategory::Dependency),
            "configuration" => Ok(RuleCategory::Configuration),
            "error_handling" => Ok(RuleCategory::ErrorHandling),
            "logging" => Ok(RuleCategory::Logging),
            "input_validation" => Ok(RuleCategory::InputValidation),
            "api_security" => Ok(RuleCategory::ApiSecurity),
            "hardcoded_secrets" => Ok(RuleCategory::HardcodedSecrets),
            "deserialization" => Ok(RuleCategory::Deserialization),
            "race_condition" => Ok(RuleCategory::RaceCondition),
            _ => Err(format!("Invalid category: {}", s)),
        }
    }
}
