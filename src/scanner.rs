use crate::config::Config;
use crate::error::{Result, CypherError};
use crate::rules::{RuleEngine, RuleResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use glob::Pattern;

/// Core scanner for traversing the codebase and executing security rules
pub struct Scanner {
    config: Config,
    rule_engine: Arc<RuleEngine>,
}

impl Scanner {
    /// Create a new scanner with configuration and a rule engine
    pub fn new(config: Config, rule_engine: RuleEngine) -> Self {
        Self {
            config,
            rule_engine: Arc::new(rule_engine),
        }
    }

    /// Scan a path (file or directory) for security issues
    pub async fn scan(&self, path: &Path) -> Result<Vec<RuleResult>> {
        if !path.exists() {
            return Err(CypherError::PathNotFound(path.display().to_string()));
        }

        // 1. Discover all files to scan
        let files = self.discover_files(path)?;

        // 2. Scan files concurrently, respecting max_threads limit
        use futures::stream::{self, StreamExt};

        let max_threads = if self.config.general.max_threads > 0 {
            self.config.general.max_threads
        } else {
            4
        };

        let rule_engine = Arc::clone(&self.rule_engine);
        
        let results_stream = stream::iter(files)
            .map(|file_path| {
                let rule_engine = Arc::clone(&rule_engine);
                async move {
                    Self::scan_single_file(&rule_engine, &file_path).await
                }
            })
            .buffer_unordered(max_threads);

        let mut all_results = Vec::new();
        let mut results_stream = Box::pin(results_stream);
        
        while let Some(res) = results_stream.next().await {
            match res {
                Ok(results) => {
                    all_results.extend(results);
                }
                Err(e) => {
                    // Log the error and continue scanning other files
                    tracing::error!("Error scanning file: {:?}", e);
                }
            }
        }

        Ok(all_results)
    }

    /// Discover files to scan in the target path
    fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if path.is_file() {
            if !self.is_excluded(path) && self.check_file_size(path)? {
                let rules = self.rule_engine.get_rules_for_file(path);
                if !rules.is_empty() {
                    files.push(path.to_path_buf());
                }
            }
            return Ok(files);
        }

        let walk = WalkDir::new(path)
            .follow_links(self.config.scanner.follow_symlinks);

        for entry in walk.into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry_path.is_file()
                && !self.is_excluded(entry_path) && self.check_file_size(entry_path)? {
                    // Only scan if the rule engine has rules for this file type
                    let rules = self.rule_engine.get_rules_for_file(entry_path);
                    if !rules.is_empty() {
                        files.push(entry_path.to_path_buf());
                    }
                }
        }

        Ok(files)
    }

    /// Check if path is excluded by configuration
    fn is_excluded(&self, path: &Path) -> bool {
        // Check relative path components to see if any are in the exclude_paths list
        for component in path.components() {
            if let Some(comp_str) = component.as_os_str().to_str() {
                if self.config.scanner.exclude_paths.iter().any(|p| p.eq_ignore_ascii_case(comp_str)) {
                    return true;
                }
            }
        }

        // Check file name against exclude patterns (globs)
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            for pattern_str in &self.config.scanner.exclude_patterns {
                if let Ok(pattern) = Pattern::new(pattern_str) {
                    if pattern.matches(file_name) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Validate that file size is within limits
    fn check_file_size(&self, path: &Path) -> Result<bool> {
        let metadata = std::fs::metadata(path)
            .map_err(CypherError::Io)?;
        Ok(metadata.len() <= self.config.scanner.max_file_size)
    }

    /// Scan a single file using the rule engine
    async fn scan_single_file(rule_engine: &RuleEngine, file_path: &Path) -> Result<Vec<RuleResult>> {
        let rules = rule_engine.get_rules_for_file(file_path);
        if rules.is_empty() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(file_path)
            .map_err(CypherError::Io)?;

        let mut results = Vec::new();
        for rule in rules {
            let result = rule_engine.run_rule(rule, &content, file_path).await?;
            if !result.passed {
                results.push(result);
            }
        }

        Ok(results)
    }
}
