use crate::error::{Result, CypherError};
use crate::rules::{Rule, RuleCategory, RuleMatch, RuleResult, Severity};
use std::collections::HashMap;
use std::path::Path;

/// Rule engine for managing and executing security rules
pub struct RuleEngine {
    /// Registered rules indexed by ID
    rules: HashMap<String, Rule>,
    /// Rules indexed by category
    rules_by_category: HashMap<RuleCategory, Vec<String>>,
    /// Rules indexed by language
    rules_by_language: HashMap<String, Vec<String>>,
    /// Current severity threshold
    severity_threshold: Severity,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            rules_by_category: HashMap::new(),
            rules_by_language: HashMap::new(),
            severity_threshold: Severity::Low,
        }
    }

    /// Create a new rule engine with a severity threshold
    pub fn with_threshold(severity_threshold: Severity) -> Self {
        Self {
            rules: HashMap::new(),
            rules_by_category: HashMap::new(),
            rules_by_language: HashMap::new(),
            severity_threshold,
        }
    }

    /// Register a rule
    pub fn register_rule(&mut self, rule: Rule) -> Result<()> {
        let rule_id = rule.id.clone();

        if self.rules.contains_key(&rule_id) {
            return Err(CypherError::Rule(format!(
                "Rule with ID '{}' already registered",
                rule_id
            )));
        }

        // Index by category
        self.rules_by_category
            .entry(rule.category)
            .or_insert_with(Vec::new)
            .push(rule_id.clone());

        // Index by language
        for language in &rule.languages {
            self.rules_by_language
                .entry(language.clone())
                .or_insert_with(Vec::new)
                .push(rule_id.clone());
        }

        self.rules.insert(rule_id, rule);
        Ok(())
    }

    /// Register multiple rules
    pub fn register_rules(&mut self, rules: Vec<Rule>) -> Result<()> {
        for rule in rules {
            self.register_rule(rule)?;
        }
        Ok(())
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.get(id)
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&Rule> {
        self.rules.values().collect()
    }

    /// Get rules by category
    pub fn get_rules_by_category(&self, category: RuleCategory) -> Vec<&Rule> {
        self.rules_by_category
            .get(&category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.rules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get rules by language
    pub fn get_rules_by_language(&self, language: &str) -> Vec<&Rule> {
        self.rules_by_language
            .get(language)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.rules.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get enabled rules
    pub fn get_enabled_rules(&self) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.enabled && r.passes_threshold(&self.severity_threshold))
            .collect()
    }

    /// Get rules that pass the severity threshold
    pub fn get_rules_by_severity(&self, severity: Severity) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.severity == severity)
            .collect()
    }

    /// Set severity threshold
    pub fn set_severity_threshold(&mut self, threshold: Severity) {
        self.severity_threshold = threshold;
    }

    /// Get current severity threshold
    pub fn severity_threshold(&self) -> &Severity {
        &self.severity_threshold
    }

    /// Enable a rule by ID
    pub fn enable_rule(&mut self, id: &str) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(id) {
            rule.enabled = true;
            Ok(())
        } else {
            Err(CypherError::Rule(format!("Rule '{}' not found", id)))
        }
    }

    /// Disable a rule by ID
    pub fn disable_rule(&mut self, id: &str) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(id) {
            rule.enabled = false;
            Ok(())
        } else {
            Err(CypherError::Rule(format!("Rule '{}' not found", id)))
        }
    }

    /// Get the number of registered rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Check if a rule is registered
    pub fn has_rule(&self, id: &str) -> bool {
        self.rules.contains_key(id)
    }

    /// Filter rules by enabled/disabled state
    pub fn filter_by_enabled(&self, enabled: bool) -> Vec<&Rule> {
        self.rules.values().filter(|r| r.enabled == enabled).collect()
    }

    /// Get rules that apply to a specific file based on language
    pub fn get_rules_for_file(&self, file_path: &Path) -> Vec<&Rule> {
        let language = self.detect_language(file_path);
        self.get_rules_by_language(language)
            .into_iter()
            .filter(|r| r.enabled && r.passes_threshold(&self.severity_threshold))
            .collect()
    }

    /// Detect programming language from file extension
    fn detect_language(&self, file_path: &Path) -> &str {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "js" | "jsx" | "mjs" => "javascript",
                "ts" | "tsx" => "typescript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "c" | "h" => "c",
                "cpp" | "hpp" | "cc" | "cxx" => "cpp",
                "cs" => "csharp",
                "php" => "php",
                "rb" => "ruby",
                "swift" => "swift",
                "kt" | "kts" => "kotlin",
                "scala" => "scala",
                _ => "unknown",
            })
            .unwrap_or("unknown")
    }

    /// Run a rule on content (placeholder for actual implementation)
    pub async fn run_rule(&self, rule: &Rule, content: &str, file_path: &Path) -> Result<RuleResult> {
        let start = std::time::Instant::now();

        // This is a placeholder - actual implementation will use the AST parser
        let matches = if let Some(pattern) = &rule.pattern {
            self.run_pattern_match(pattern, content, file_path)?
        } else {
            Vec::new()
        };

        let duration = start.elapsed().as_millis() as u64;
        Ok(RuleResult::new(rule.clone(), matches, duration))
    }

    /// Run pattern matching (simple regex-based for now)
    fn run_pattern_match(&self, pattern: &str, content: &str, file_path: &Path) -> Result<Vec<RuleMatch>> {
        let regex = regex::Regex::new(pattern)
            .map_err(|e| CypherError::Rule(format!("Invalid pattern '{}': {}", pattern, e)))?;

        let mut matches = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(RuleMatch {
                    file: file_path.to_path_buf(),
                    line: line_num + 1,
                    column: mat.start() + 1,
                    snippet: mat.as_str().to_string(),
                    context: line.to_string(),
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                });
            }
        }

        Ok(matches)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_engine_creation() {
        let engine = RuleEngine::new();
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn test_register_rule() {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            "TEST-001".to_string(),
            "Test Rule".to_string(),
            "A test rule".to_string(),
            Severity::Low,
            RuleCategory::Configuration,
            vec!["rust".to_string()],
        );

        engine.register_rule(rule).unwrap();
        assert_eq!(engine.rule_count(), 1);
        assert!(engine.has_rule("TEST-001"));
    }

    #[test]
    fn test_duplicate_rule() {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            "TEST-001".to_string(),
            "Test Rule".to_string(),
            "A test rule".to_string(),
            Severity::Low,
            RuleCategory::Configuration,
            vec!["rust".to_string()],
        );

        engine.register_rule(rule.clone()).unwrap();
        assert!(engine.register_rule(rule).is_err());
    }

    #[test]
    fn test_severity_threshold() {
        let engine = RuleEngine::with_threshold(Severity::High);
        assert_eq!(engine.severity_threshold(), &Severity::High);
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut engine = RuleEngine::new();
        let rule = Rule::new(
            "TEST-001".to_string(),
            "Test Rule".to_string(),
            "A test rule".to_string(),
            Severity::Low,
            RuleCategory::Configuration,
            vec!["rust".to_string()],
        );

        engine.register_rule(rule).unwrap();
        engine.disable_rule("TEST-001").unwrap();
        assert!(!engine.get_rule("TEST-001").unwrap().enabled);

        engine.enable_rule("TEST-001").unwrap();
        assert!(engine.get_rule("TEST-001").unwrap().enabled);
    }
}
