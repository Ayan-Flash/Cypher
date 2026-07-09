use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_runs() {
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cypher.toml");

    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(config_path.exists());
    
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[general]"));
    assert!(content.contains("[scanner]"));
    assert!(content.contains("[rules]"));
}

#[test]
fn test_validate_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cypher.toml");

    // Create a valid config
    let config_content = r#"
[general]
verbose = false
color = true
max_threads = 4

[scanner]
exclude_paths = ["node_modules", "target"]
exclude_patterns = ["*.min.js"]
max_file_size = 10485760
follow_symlinks = false

[rules]
severity_threshold = "low"
enabled_rules = []
disabled_rules = []
enabled_categories = []

[reporting]
format = "text"
include_snippets = true
max_issues = 1000

[plugins]
directory = ".cypher/plugins"
auto_load = true
"#;

    fs::write(&config_path, config_content).unwrap();

    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("validate")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_list_rules_command() {
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("list-rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available security rules"));
}

#[test]
fn test_scan_command_help() {
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("scan")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cypher"));
}

#[test]
fn test_scan_finds_secrets_and_fails() {
    let temp_dir = TempDir::new().unwrap();
    let code_file = temp_dir.path().join("app.js");

    // Write a JS file containing a hardcoded API key
    let content = r#"
        const API_KEY = "apikey = 'abc123xyz78901234567'";
        console.log("App started");
    "#;
    fs::write(&code_file, content).unwrap();

    // Run scan command on temp_dir
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("--fail-on-issues")
        .assert()
        .failure()
        .stdout(predicate::str::contains("SEC-001"))
        .stdout(predicate::str::contains("Hardcoded API Key"));
}

#[test]
fn test_scan_respects_exclusions() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create node_modules directory which should be excluded
    let node_modules_dir = temp_dir.path().join("node_modules");
    fs::create_dir_all(&node_modules_dir).unwrap();
    let excluded_file = node_modules_dir.join("bad.js");
    fs::write(&excluded_file, "const API_KEY = 'apikey = \"abc123xyz78901234567\"';").unwrap();

    // Create a regular directory that should be scanned
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let scanned_file = src_dir.join("app.js");
    fs::write(&scanned_file, "const API_KEY = 'apikey = \"abc123xyz78901234567\"';").unwrap();

    // Run scan, check that only the file in src is reported, but not node_modules
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("-o")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("src").and(predicate::str::contains("app.js")))
        .stdout(predicate::str::contains("node_modules").not());
}

#[test]
fn test_list_rules_with_filters() {
    // Filter by severity critical
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("list-rules")
        .arg("--severity")
        .arg("critical")
        .assert()
        .success()
        .stdout(predicate::str::contains("CRITICAL"))
        .stdout(predicate::str::contains("LOW").not());
}

#[test]
fn test_ask_command_requires_api_key() {
    // Since local validation is removed, both queries pass validation but fail on missing API key
    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("ask")
        .arg("What is the capital of Japan?")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key is required"));

    Command::cargo_bin("cypher-cli")
        .unwrap()
        .arg("ask")
        .arg("How do I prevent SQL Injection in Rust?")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key is required"));
}
