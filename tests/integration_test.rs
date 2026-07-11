use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_runs() {
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cypher.toml");

    Command::cargo_bin("cypher")
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

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("validate")
        .arg(&config_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_list_rules_command() {
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("list-rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available security rules"));
}

#[test]
fn test_scan_command_help() {
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("scan")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("cypher")
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
    Command::cargo_bin("cypher")
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
    Command::cargo_bin("cypher")
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
    Command::cargo_bin("cypher")
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
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("ask")
        .arg("What is the capital of Japan?")
        .assert()
        .failure();

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("ask")
        .arg("How do I prevent SQL Injection in Rust?")
        .assert()
        .failure();
}

#[test]
fn test_fix_dry_run_finds_nothing() {
    let temp_dir = TempDir::new().unwrap();

    // Create a clean file with no issues
    let clean_file = temp_dir.path().join("clean.js");
    fs::write(&clean_file, "const x = 1; console.log(x);").unwrap();

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("fix")
        .arg("--dry-run")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No security issues found"));
}

#[test]
fn test_report_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let code_file = temp_dir.path().join("app.js");
    fs::write(&code_file, "const API_KEY = 'apikey = \"abc123xyz78901234567\"';").unwrap();

    let report_file = temp_dir.path().join("report.json");

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("report")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&report_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Report generated successfully"));

    assert!(report_file.exists());
    let content = fs::read_to_string(&report_file).unwrap();
    assert!(content.contains("findings"));
    assert!(content.contains("cypher"));
    assert!(content.contains("SEC-001"));
}

#[test]
fn test_scan_with_specific_rules() {
    let temp_dir = TempDir::new().unwrap();
    let code_file = temp_dir.path().join("app.js");
    fs::write(&code_file, r#"
        const API_KEY = "apikey = 'abc123xyz78901234567'";
        const password = "mypassword123";
    "#).unwrap();

    // Only run a single rule
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("--rules")
        .arg("SEC-001")
        .arg("-o")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("SEC-001"))
        .stdout(predicate::str::contains("SEC-002").not());
}

#[test]
fn test_scan_with_exclude_rules() {
    let temp_dir = TempDir::new().unwrap();
    let code_file = temp_dir.path().join("app.js");
    fs::write(&code_file, r#"
        const password = "mypassword123";
    "#).unwrap();

    // Run scan excluding the password rule
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("--exclude-rules")
        .arg("SEC-002")
        .arg("-o")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_scan_respects_max_issues() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple files with issues
    for i in 0..5 {
        let file = temp_dir.path().join(format!("app{}.js", i));
        fs::write(&file, format!("const KEY_{} = \"apikey = 'abc123xyz78901234567'\";", i)).unwrap();
    }

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("--max-issues")
        .arg("2")
        .arg("-o")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_init_force_overwrite() {
    let temp_dir = TempDir::new().unwrap();

    // First init
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Second init without force should fail
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .failure();

    // Second init with force should succeed
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("init")
        .arg("--force")
        .current_dir(temp_dir.path())
        .assert()
        .success();
}

#[test]
fn test_validate_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cypher.toml");

    let invalid_config = r#"
[general]
max_threads = 0
    "#;

    fs::write(&config_path, invalid_config).unwrap();

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("validate")
        .arg(&config_path)
        .assert()
        .failure();
}

#[test]
fn test_list_rules_by_category() {
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("list-rules")
        .arg("--category")
        .arg("hardcoded_secrets")
        .assert()
        .success()
        .stdout(predicate::str::contains("SEC-001")
            .and(predicate::str::contains("SEC-015"))
            .and(predicate::str::contains("SEC-005").not()));
}

#[test]
fn test_scan_ignores_comments() {
    let temp_dir = TempDir::new().unwrap();
    let code_file = temp_dir.path().join("app.js");

    // Write a JS file containing a commented out password and API key, and a live one.
    // The commented ones should be ignored, and the live one should be caught.
    let content = r#"
        // const API_KEY = "apikey = 'abc123xyz78901234567'";
        /* const password = "mypassword123"; */
        const API_KEY = "apikey = 'realapikey1234567890'";
    "#;
    fs::write(&code_file, content).unwrap();

    // Run scan command on temp_dir
    Command::cargo_bin("cypher")
        .unwrap()
        .arg("scan")
        .arg(temp_dir.path())
        .arg("-o")
        .arg("json")
        .assert()
        .success()
        // Should catch the live API_KEY
        .stdout(predicate::str::contains("realapikey1234567890"))
        // Should NOT catch the commented out password (SEC-002)
        .stdout(predicate::str::contains("mypassword123").not())
        // Should NOT report SEC-002 (password rule) as triggered
        .stdout(predicate::str::contains("SEC-002").not());
}

#[test]
fn test_error_output_is_display_not_debug() {
    // Regression test: top-level failures must print the clean, human-readable
    // Display message (e.g. "Error: Configuration error: ...") and never Rust's
    // raw Debug-formatted enum/tuple syntax (e.g. `Config("...")`).
    let temp_dir = TempDir::new().unwrap();
    let missing_config = temp_dir.path().join("missing.toml");

    Command::cargo_bin("cypher")
        .unwrap()
        .arg("validate")
        .arg(&missing_config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Configuration error:"))
        .stderr(predicate::str::contains("Config(\"").not());
}

#[test]
fn test_upgrade_command() {
    let assert = Command::cargo_bin("cypher")
        .unwrap()
        .arg("upgrade")
        .assert();

    // It should output version checking logs
    assert
        .stdout(predicate::str::contains("Checking for updates from GitHub Releases"));
}
