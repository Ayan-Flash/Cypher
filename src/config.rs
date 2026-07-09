use crate::error::{Result, CypherError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Cypher configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,
    /// Scanner settings
    #[serde(default)]
    pub scanner: ScannerConfig,
    /// Rule settings
    #[serde(default)]
    pub rules: RulesConfig,
    /// Reporting settings
    #[serde(default)]
    pub reporting: ReportingConfig,
    /// Plugin settings
    #[serde(default)]
    pub plugins: PluginsConfig,
    /// AI settings
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Verbose output
    #[serde(default = "default_false")]
    pub verbose: bool,
    /// Colored output
    #[serde(default = "default_true")]
    pub color: bool,
    /// Maximum number of concurrent threads
    #[serde(default = "default_threads")]
    pub max_threads: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Paths to exclude from scanning
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    /// File patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Maximum file size to scan (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    /// Follow symbolic links
    #[serde(default = "default_false")]
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig {
    /// Severity threshold
    #[serde(default = "default_severity")]
    pub severity_threshold: String,
    /// Specific rules to enable
    #[serde(default)]
    pub enabled_rules: Vec<String>,
    /// Rules to disable
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    /// Rule categories to enable
    #[serde(default)]
    pub enabled_categories: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Default output format
    #[serde(default = "default_output_format")]
    pub format: String,
    /// Include source code snippets in reports
    #[serde(default = "default_true")]
    pub include_snippets: bool,
    /// Maximum number of issues to report
    #[serde(default = "default_max_issues")]
    pub max_issues: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Plugin directory
    #[serde(default = "default_plugin_dir")]
    pub directory: String,
    /// Auto-load plugins
    #[serde(default = "default_true")]
    pub auto_load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// API key for the AI provider
    pub api_key: Option<String>,
    /// Provider name (e.g. "gemini")
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Model name (e.g. "gemini-1.5-flash")
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_provider() -> String {
    "gemini".to_string()
}

fn default_model() -> String {
    "gemini-3.1-pro".to_string()
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            provider: default_provider(),
            model: default_model(),
        }
    }
}

// Default value functions
fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

fn default_threads() -> usize {
    4
}

fn default_max_file_size() -> u64 {
    10 * 1024 * 1024 // 10MB
}

fn default_severity() -> String {
    "low".to_string()
}

fn default_output_format() -> String {
    "text".to_string()
}

fn default_max_issues() -> usize {
    1000
}

fn default_plugin_dir() -> String {
    ".cypher/plugins".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                verbose: false,
                color: true,
                max_threads: 4,
            },
            scanner: ScannerConfig {
                exclude_paths: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    "vendor".to_string(),
                    ".git".to_string(),
                ],
                exclude_patterns: vec!["*.min.js".to_string(), "*.min.css".to_string()],
                max_file_size: 10 * 1024 * 1024,
                follow_symlinks: false,
            },
            rules: RulesConfig {
                severity_threshold: "low".to_string(),
                enabled_rules: vec![],
                disabled_rules: vec![],
                enabled_categories: vec![],
            },
            reporting: ReportingConfig {
                format: "text".to_string(),
                include_snippets: true,
                max_issues: 1000,
            },
            plugins: PluginsConfig {
                directory: ".cypher/plugins".to_string(),
                auto_load: true,
            },
            ai: AiConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CypherError::Config(format!("Failed to read config file: {}", e)))?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| CypherError::Config("Invalid config file extension".to_string()))?;

        let config = match extension {
            "toml" => {
                toml::from_str(&content)
                    .map_err(|e| CypherError::Config(format!("Failed to parse TOML: {}", e)))?
            }
            "yaml" | "yml" => {
                yaml_rust2::YamlLoader::load_from_str(&content)
                    .map_err(|e| CypherError::Config(format!("Failed to parse YAML: {}", e)))?;
                serde_yaml::from_str(&content)
                    .map_err(|e| CypherError::Config(format!("Failed to parse YAML: {}", e)))?
            }
            "json" => {
                serde_json::from_str(&content)
                    .map_err(|e| CypherError::Config(format!("Failed to parse JSON: {}", e)))?
            }
            _ => {
                return Err(CypherError::Config(format!(
                    "Unsupported config format: {}",
                    extension
                )))
            }
        };

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("toml");

        let content = match extension {
            "toml" => toml::to_string_pretty(self)
                .map_err(|e| CypherError::Config(format!("Failed to serialize TOML: {}", e)))?,
            "yaml" | "yml" => serde_yaml::to_string(self)
                .map_err(|e| CypherError::Config(format!("Failed to serialize YAML: {}", e)))?,
            "json" => serde_json::to_string_pretty(self)
                .map_err(|e| CypherError::Config(format!("Failed to serialize JSON: {}", e)))?,
            _ => {
                return Err(CypherError::Config(format!(
                    "Unsupported config format: {}",
                    extension
                )))
            }
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CypherError::Config(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, content)
            .map_err(|e| CypherError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Find and load configuration from default locations
    pub fn load_default() -> Result<Self> {
        let mut default_paths = vec![
            PathBuf::from("cypher.toml"),
            PathBuf::from("cypher.yaml"),
            PathBuf::from("cypher.yml"),
            PathBuf::from("cypher.json"),
            PathBuf::from(".cypher/config.toml"),
            PathBuf::from(".cypher/config.yaml"),
            PathBuf::from(".cypher/config.yml"),
            PathBuf::from(".cypher/config.json"),
        ];

        // Add global home configuration paths
        if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from) {
            default_paths.push(home.join(".cypher").join("settings.json"));
            default_paths.push(home.join(".cypher").join("config.toml"));
            default_paths.push(home.join(".cypher").join("config.json"));
        }

        for path in default_paths {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        Ok(Config::default())
    }

    /// Retrieve the API key for a provider, trying:
    /// 1. Environment variables (e.g. GEMINI_API_KEY)
    /// 2. OS credential store (Keyring)
    /// 3. Plain-text configuration file (fallback)
    pub fn get_secure_api_key(&self, provider: &str) -> Option<String> {
        // 1. Check environment variables first
        let env_var_name = match provider.to_lowercase().as_str() {
            "gemini" => "GEMINI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            _ => "",
        };

        if !env_var_name.is_empty() {
            if let Ok(key) = std::env::var(env_var_name) {
                if !key.trim().is_empty() {
                    return Some(key.trim().to_string());
                }
            }
        }

        // 2. Try OS Credential Store (Keyring)
        let entry_name = format!("cypher-cli-{}", provider.to_lowercase());
        if let Ok(entry) = keyring::Entry::new(&entry_name, "user") {
            if let Ok(password) = entry.get_password() {
                if !password.trim().is_empty() {
                    return Some(password.trim().to_string());
                }
            }
        }

        // 3. Fall back to configuration file (plain text)
        if self.ai.provider.eq_ignore_ascii_case(provider) {
            if let Some(ref key) = self.ai.api_key {
                if !key.trim().is_empty() {
                    return Some(key.trim().to_string());
                }
            }
        }

        // Also check if any key exists in the config's ai section anyway
        if let Some(ref key) = self.ai.api_key {
            if !key.trim().is_empty() {
                return Some(key.trim().to_string());
            }
        }

        None
    }

    /// Save the API key securely. Try saving to Keyring first.
    /// If Keyring fails, save to configuration file.
    pub fn save_secure_api_key(provider: &str, key: &str) -> Result<()> {
        let key = key.trim();
        if key.is_empty() {
            return Err(CypherError::Config("API key cannot be empty".to_string()));
        }

        let entry_name = format!("cypher-cli-{}", provider.to_lowercase());
        let keyring_result = keyring::Entry::new(&entry_name, "user")
            .and_then(|entry| entry.set_password(key));

        match keyring_result {
            Ok(_) => Ok(()),
            Err(e) => Err(CypherError::Config(format!("{}", e))),
        }
    }



    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let valid_severities = ["low", "medium", "high", "critical"];
        if !valid_severities.contains(&self.rules.severity_threshold.as_str()) {
            return Err(CypherError::Config(format!(
                "Invalid severity threshold: {}",
                self.rules.severity_threshold
            )));
        }

        let valid_formats = ["text", "json", "sarif", "html"];
        if !valid_formats.contains(&self.reporting.format.as_str()) {
            return Err(CypherError::Config(format!(
                "Invalid output format: {}",
                self.reporting.format
            )));
        }

        if self.general.max_threads == 0 {
            return Err(CypherError::Config(
                "max_threads must be greater than 0".to_string(),
            ));
        }

        let valid_providers = ["gemini", "anthropic", "openai", "openrouter"];
        if !valid_providers.contains(&self.ai.provider.as_str()) {
            return Err(CypherError::Config(format!(
                "Invalid AI provider: {}. Valid providers are 'gemini', 'anthropic', 'openai', and 'openrouter'.",
                self.ai.provider
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ai_config_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("cypher.json");

        let json_content = r#"{
            "general": {
                "verbose": true,
                "color": false,
                "max_threads": 8
            },
            "scanner": {
                "exclude_paths": ["node_modules"],
                "exclude_patterns": ["*.min.js"],
                "max_file_size": 1048576,
                "follow_symlinks": true
            },
            "rules": {
                "severity_threshold": "high",
                "enabled_rules": ["SEC-001"],
                "disabled_rules": [],
                "enabled_categories": []
            },
            "reporting": {
                "format": "json",
                "include_snippets": false,
                "max_issues": 100
            },
            "plugins": {
                "directory": "custom/plugins",
                "auto_load": false
            },
            "ai": {
                "api_key": "some-secret-api-key",
                "provider": "gemini",
                "model": "gemini-1.5-pro"
            }
        }"#;

        std::fs::write(&config_path, json_content).unwrap();

        let config = Config::load_from_file(&config_path).unwrap();
        assert!(config.general.verbose);
        assert!(!config.general.color);
        assert_eq!(config.general.max_threads, 8);
        assert_eq!(config.ai.api_key.as_deref(), Some("some-secret-api-key"));
        assert_eq!(config.ai.provider, "gemini");
        assert_eq!(config.ai.model, "gemini-1.5-pro");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ai_config_validation() {
        let mut config = Config::default();
        config.ai.provider = "unsupported-provider".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_openai_provider_valid() {
        let mut config = Config::default();
        config.ai.provider = "openai".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_multiple_config_formats() {
        use tempfile::TempDir;

        // Test YAML config
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("cypher.yaml");
        let yaml_content = r#"
general:
  verbose: true
  color: false
  max_threads: 8
ai:
  provider: openai
  model: gpt-4o
"#;
        std::fs::write(&yaml_path, yaml_content).unwrap();
        let config = Config::load_from_file(&yaml_path).unwrap();
        assert_eq!(config.ai.provider, "openai");
        assert_eq!(config.ai.model, "gpt-4o");
        assert!(config.general.verbose);

        // Test TOML config
        let toml_path = temp_dir.path().join("cypher.toml");
        let toml_content = r#"
[general]
verbose = true
color = true
max_threads = 4

[ai]
provider = "anthropic"
model = "claude-3-5-sonnet-20240620"
"#;
        std::fs::write(&toml_path, toml_content).unwrap();
        let config = Config::load_from_file(&toml_path).unwrap();
        assert_eq!(config.ai.provider, "anthropic");
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.max_threads, 4);
        assert_eq!(config.scanner.max_file_size, 10 * 1024 * 1024);
        assert!(config.general.color);
        assert!(!config.general.verbose);
    }
}

