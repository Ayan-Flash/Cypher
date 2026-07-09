#![allow(dead_code)]

use crate::rules::{RuleCategory, Severity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single security rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Category
    pub category: RuleCategory,
    /// CWE identifier (if applicable)
    pub cwe: Option<String>,
    /// OWASP reference (if applicable)
    pub owasp: Option<String>,
    /// Supported languages
    pub languages: Vec<String>,
    /// Whether the rule is enabled by default
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Pattern or logic to match
    pub pattern: Option<String>,
    /// Custom rule metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_true() -> bool {
    true
}

/// Represents a match found by a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    /// File where the match was found
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Matched code snippet
    pub snippet: String,
    /// Context around the match
    pub context: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Result of running a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// The rule that was run
    pub rule: Rule,
    /// Matches found
    pub matches: Vec<RuleMatch>,
    /// Whether the rule passed (no matches)
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl Rule {
    /// Create a new rule
    pub fn new(
        id: String,
        name: String,
        description: String,
        severity: Severity,
        category: RuleCategory,
        languages: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            severity,
            category,
            cwe: None,
            owasp: None,
            languages,
            enabled: true,
            pattern: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add CWE identifier
    pub fn with_cwe(mut self, cwe: String) -> Self {
        self.cwe = Some(cwe);
        self
    }

    /// Add OWASP reference
    pub fn with_owasp(mut self, owasp: String) -> Self {
        self.owasp = Some(owasp);
        self
    }

    /// Add pattern
    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Check if rule applies to a language
    pub fn applies_to_language(&self, language: &str) -> bool {
        self.languages.is_empty() || self.languages.iter().any(|l| {
            l.to_lowercase() == language.to_lowercase() || l == "*"
        })
    }

    /// Check if rule passes severity threshold
    pub fn passes_threshold(&self, threshold: &Severity) -> bool {
        self.severity >= *threshold
    }
}

impl RuleResult {
    /// Create a new rule result
    pub fn new(rule: Rule, matches: Vec<RuleMatch>, duration_ms: u64) -> Self {
        let passed = matches.is_empty();
        Self {
            rule,
            matches,
            passed,
            duration_ms,
        }
    }

    /// Get the number of findings
    pub fn finding_count(&self) -> usize {
        self.matches.len()
    }
}
