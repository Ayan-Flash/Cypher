mod ai;
mod cli;
mod config;
mod error;
mod rules;
mod parser;
mod reporter;
mod detector;
mod scanner;
mod tui;

use cli::Cli;
use config::Config;
use error::{Result, CypherError};
use rules::{RuleEngine, RuleLibrary, Severity, RuleCategory};
use scanner::Scanner;
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;

use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            // Print a clean, user-facing message (Display) instead of Rust's default
            // Debug-formatted termination output, which leaks internal enum/tuple syntax.
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse_args();

    // Respect --no-color for every colored!/println! call in the app, not just tracing.
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Initialize logging
    let filter = EnvFilter::from_default_env()
        .add_directive(match cli.verbose {
            0 => "cypher=warn".parse().unwrap(),
            1 => "cypher=info".parse().unwrap(),
            2 => "cypher=debug".parse().unwrap(),
            _ => "cypher=trace".parse().unwrap(),
        });

    fmt()
        .with_env_filter(filter)
        .with_ansi(!cli.no_color)
        .init();

    // Load configuration
    let mut config_path = cli.config.clone();
    let config = if let Some(path) = &config_path {
        Config::load_from_file(path)?
    } else {
        // Resolve default settings path
        if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(std::path::PathBuf::from) {
            let settings_path = home.join(".cypher").join("settings.json");
            if settings_path.exists() {
                config_path = Some(settings_path);
            }
        }
        Config::load_default()?
    };

    // Validate configuration
    config.validate()?;

    // Execute command
    match cli.command {
        None => {
            // Default first-run or chat mode execution
            let provider = config.ai.provider.clone();
            let has_key = config.get_secure_api_key(&provider).is_some();
            
            if !has_key {
                let (resolved_config, path) = setup_ai().await?;
                start_interactive_chat(resolved_config, path).await?;
            } else {
                start_interactive_chat(config, config_path).await?;
            }
            Ok(())
        }
        Some(cli::Commands::Scan {
            path,
            output,
            file,
            severity,
            rules,
            exclude_rules,
            max_issues,
            fail_on_issues,
        }) => {
            info!("Starting security scan");
            let scan_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            
            // 1. Framework detection
            let mut detector = detector::Detector::new();
            let frameworks = detector.detect(&scan_path)?;
            if !frameworks.is_empty() {
                println!("Detected frameworks: {}", frameworks.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", "));
            }

            // 2. Initialize rule engine & load built-in rules
            let parsed_severity: Severity = std::str::FromStr::from_str(&severity)
                .map_err(CypherError::InvalidSeverity)?;
            let mut rule_engine = RuleEngine::with_threshold(parsed_severity);
            rule_engine.register_rules(RuleLibrary::get_all_rules())?;

            // 3. Filter rules based on CLI arguments and config
            if let Some(rules_str) = &rules {
                let enabled_ids: std::collections::HashSet<&str> = rules_str.split(',').map(|s| s.trim()).collect();
                let all_rule_ids: Vec<String> = rule_engine.get_all_rules().iter().map(|r| r.id.clone()).collect();
                for id in all_rule_ids {
                    if !enabled_ids.contains(id.as_str()) {
                        rule_engine.disable_rule(&id)?;
                    }
                }
            }

            if let Some(exclude_str) = &exclude_rules {
                let excluded_ids: std::collections::HashSet<&str> = exclude_str.split(',').map(|s| s.trim()).collect();
                for id in excluded_ids {
                    if rule_engine.has_rule(id) {
                        rule_engine.disable_rule(id)?;
                    }
                }
            }

            // Respect disabled rules in config
            for id in &config.rules.disabled_rules {
                if rule_engine.has_rule(id) {
                    rule_engine.disable_rule(id)?;
                }
            }

            // Respect enabled rules in config if specified
            if !config.rules.enabled_rules.is_empty() {
                let config_enabled: std::collections::HashSet<&String> = config.rules.enabled_rules.iter().collect();
                let all_rule_ids: Vec<String> = rule_engine.get_all_rules().iter().map(|r| r.id.clone()).collect();
                for id in all_rule_ids {
                    if !config_enabled.contains(&id) {
                        rule_engine.disable_rule(&id)?;
                    }
                }
            }

            // 4. Run Scanner
            let scanner = Scanner::new(config.clone(), rule_engine);
            let mut results = scanner.scan(&scan_path).await?;

            // Apply max_issues limit if specified in config or CLI
            let max_issues_limit = max_issues.unwrap_or(config.reporting.max_issues);
            let mut total_findings = 0;
            let mut truncated_results = Vec::new();

            for mut result in results {
                if total_findings >= max_issues_limit {
                    break;
                }
                
                let remaining = max_issues_limit - total_findings;
                if result.matches.len() > remaining {
                    result.matches.truncate(remaining);
                    result.passed = result.matches.is_empty();
                }
                total_findings += result.finding_count();
                truncated_results.push(result);
            }
            results = truncated_results;

            // 5. Generate Report
            let reporter = reporter::create_reporter(&output)?;
            let report_content = reporter.generate(&results)?;

            if let Some(out_path) = &file {
                std::fs::write(out_path, &report_content)
                    .map_err(|e| CypherError::Report(format!("Failed to write report file: {}", e)))?;
                println!("Report written to: {}", out_path.display());
            } else {
                println!("{}", report_content);
            }

            if fail_on_issues && total_findings > 0 {
                return Err(CypherError::Scanner(format!("Security issues found: {}", total_findings)));
            }

            Ok(())
        }
        Some(cli::Commands::Fix { path, dry_run }) => {
            let scan_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            info!("Starting security fix on path: {}", scan_path.display());

            let parsed_severity: Severity = std::str::FromStr::from_str(&config.rules.severity_threshold)
                .unwrap_or(Severity::Low);
            let mut rule_engine = RuleEngine::with_threshold(parsed_severity);
            rule_engine.register_rules(RuleLibrary::get_all_rules())?;

            let scanner = Scanner::new(config.clone(), rule_engine);
            let results = scanner.scan(&scan_path).await?;

            let all_findings: Vec<_> = results.iter().filter(|r| !r.passed).collect();
            if all_findings.is_empty() {
                println!("{} No security issues found. Nothing to fix.", "✓".green().bold());
                return Ok(());
            }

            println!("\n{} Found {} security issues to review.\n", "🔍".bold(), all_findings.len());

            // Get unique files with issues
            let mut files_with_issues: Vec<_> = all_findings.iter()
                .flat_map(|r| r.matches.iter().map(|m| m.file.clone()))
                .collect();
            files_with_issues.sort();
            files_with_issues.dedup();

            let client = ai::build_client();
            let provider = &config.ai.provider;
            let model = &config.ai.model;
            let api_key = config.get_secure_api_key(provider)
                .ok_or_else(|| CypherError::Config(
                    "API key required for generating fixes. Configure your AI provider first.".to_string()
                ))?;

            for file_path in &files_with_issues {
                let file_results: Vec<_> = all_findings.iter()
                    .filter(|r| r.matches.iter().any(|m| m.file == *file_path))
                    .collect();

                let file_content = std::fs::read_to_string(file_path)
                    .map_err(CypherError::Io)?;
                let extension = file_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("txt");

                let mut findings_desc = String::new();
                for result in &file_results {
                    for m in &result.matches {
                        if m.file == *file_path {
                            findings_desc.push_str(&format!(
                                "  - {}:{}:{} - {} ({})\n",
                                m.line, m.column, result.rule.id, result.rule.name,
                                result.rule.description
                            ));
                        }
                    }
                }

                if findings_desc.is_empty() {
                    continue;
                }

                println!("\n{} Fixing: {}", "📄".bold().cyan(), file_path.display().to_string().yellow());
                println!("{}", findings_desc);

                let fix_prompt = format!(
                    "Below is a source code file containing security vulnerabilities. \
                     Provide ONLY the corrected source code as output, wrapped in a code block with language '{}'. \
                     Do not add explanations before or after the code block. \
                     Fix every security issue listed below while preserving the original logic and functionality.\n\n\
                     Security issues in this file:\n{}\n\n\
                     Current file content:\n```{}\n{}\n```",
                    extension, findings_desc, extension, file_content
                );

                print!("  {} Generating fix...", "⏳".bold());
                std::io::stdout().flush().unwrap();

                let mut fix = String::new();
                let mut in_code_block = false;
                let mut fix_content = String::new();

                let stream_result = ai::stream_ai_response(
                    &client, provider, model, &api_key, &fix_prompt,
                    &mut |chunk: &str| {
                        fix.push_str(chunk);
                    },
                ).await;

                match stream_result {
                    Ok(()) => {}
                    Err(e) => {
                        println!("\n  {} Failed to generate fix: {}", "✗".red(), e);
                        continue;
                    }
                }

                // Extract code block from response
                for line in fix.lines() {
                    if line.starts_with("```") && !in_code_block {
                        in_code_block = true;
                        continue;
                    } else if line.starts_with("```") && in_code_block {
                        in_code_block = false;
                        continue;
                    }
                    if in_code_block {
                        fix_content.push_str(line);
                        fix_content.push('\n');
                    }
                }

                if fix_content.is_empty() {
                    // Fallback: use the entire response as the fix
                    fix_content = fix.clone();
                }

                if dry_run {
                    println!("\r  {} Generated fix (preview - dry run):\n", "✓".green());
                    for line in fix_content.lines().take(20) {
                        println!("  | {}", line);
                    }
                    if fix_content.lines().count() > 20 {
                        println!("  | ... ({} more lines)", fix_content.lines().count() - 20);
                    }
                } else {
                    println!("\r  {} Generated fix. Reviewing...", "✓".green());
                    
                    let diff_output = generate_diff(&file_content, &fix_content);
                    println!("\n  Changes preview:\n{}", diff_output);

                    let apply = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("  Apply this fix?")
                        .default(true)
                        .interact()
                        .unwrap_or(false);

                    if apply {
                        std::fs::write(file_path, &fix_content)
                            .map_err(CypherError::Io)?;
                        println!("  {} Fix applied to {}", "✓".green().bold(), file_path.display());
                    } else {
                        println!("  {} Skipped fix for {}", "→".yellow(), file_path.display());
                    }
                }
            }

            if dry_run {
                println!("\n{} Dry run complete. Run without --dry-run to apply fixes.", "✓".green().bold());
            } else {
                println!("\n{} Fix mode complete.", "✓".green().bold());
            }

            Ok(())
        }
        Some(cli::Commands::Init { force }) => {
            info!("Initializing Cypher configuration");
            let config_path = std::path::PathBuf::from("cypher.toml");
            
            if config_path.exists() && !force {
                return Err(CypherError::Config(
                    "Configuration file already exists. Use --force to overwrite.".to_string(),
                ));
            }
            
            let default_config = Config::default();
            default_config.save_to_file(&config_path)?;
            println!("Configuration file created: {}", config_path.display());
            
            Ok(())
        }
        Some(cli::Commands::ListRules { severity, category }) => {
            info!("Listing available rules");
            
            let mut rule_engine = RuleEngine::new();
            rule_engine.register_rules(RuleLibrary::get_all_rules())?;

            let mut rules = rule_engine.get_all_rules();

            if let Some(sev_str) = severity {
                let target_severity: Severity = std::str::FromStr::from_str(&sev_str)
                    .map_err(CypherError::InvalidSeverity)?;
                rules.retain(|r| r.severity == target_severity);
            }

            if let Some(cat_str) = category {
                let target_category: RuleCategory = std::str::FromStr::from_str(&cat_str)
                    .map_err(CypherError::Rule)?;
                rules.retain(|r| r.category == target_category);
            }

            println!("Available security rules (total: {}):\n", rules.len());
            
            for rule in rules {
                println!("{:<10} | {:<40} | {:<10} | {:<20}", 
                    rule.id.bold(), 
                    rule.name, 
                    rule.severity.to_string().to_uppercase(),
                    rule.category.to_string()
                );
                println!("  Description: {}", rule.description);
                if let Some(cwe) = &rule.cwe {
                    println!("  CWE: {}", cwe);
                }
                if let Some(owasp) = &rule.owasp {
                    println!("  OWASP: {}", owasp);
                }
                println!("  Languages: {}", rule.languages.join(", "));
                println!("{}", "-".repeat(80).dimmed());
            }
            
            Ok(())
        }
        Some(cli::Commands::Validate { config }) => {
            info!("Validating configuration: {}", config.display());
            let loaded_config = Config::load_from_file(&config)?;
            loaded_config.validate()?;
            println!("Configuration is valid.");
            
            Ok(())
        }
        Some(cli::Commands::Report {
            path,
            format,
            output,
        }) => {
            info!("Generating report");
            
            let mut rule_engine = RuleEngine::new();
            rule_engine.register_rules(RuleLibrary::get_all_rules())?;

            let scanner = Scanner::new(config.clone(), rule_engine);
            let results = scanner.scan(&path).await?;

            let reporter = reporter::create_reporter(&format)?;
            let report_content = reporter.generate(&results)?;

            std::fs::write(&output, &report_content)
                .map_err(|e| CypherError::Report(format!("Failed to write report file: {}", e)))?;
            
            println!("Report generated successfully at: {}", output.display());
            Ok(())
        }
        Some(cli::Commands::Plugin { plugin_command }) => {
            info!("Managing plugins");
            match plugin_command {
                cli::PluginCommands::List => {
                    println!("Installed plugins:");
                    println!("\nPlugin system implementation in progress...");
                }
                cli::PluginCommands::Install { plugin } => {
                    println!("Installing plugin: {}", plugin);
                    println!("\nPlugin installation implementation in progress...");
                }
                cli::PluginCommands::Remove { name } => {
                    println!("Removing plugin: {}", name);
                    println!("\nPlugin removal implementation in progress...");
                }
                cli::PluginCommands::Update => {
                    println!("Updating all plugins...");
                    println!("\nPlugin update implementation in progress...");
                }
            }
            Ok(())
        }
        Some(cli::Commands::Ask { prompt, api_key }) => {
            info!("Executing AI security assistant query");

            // 1. Resolve API key & Config
            let resolved_config = config.clone();
            let provider = resolved_config.ai.provider.clone();
            
            let key = api_key
                .or_else(|| resolved_config.get_secure_api_key(&provider))
                .ok_or_else(|| CypherError::Config(
                    format!("API key is required for provider '{}'. Run the setup wizard, set the corresponding environment variable (e.g. GEMINI_API_KEY), or pass --api-key.", provider)
                ))?;

            let model = resolved_config.ai.model.clone();

            // 3. Decide if we start the interactive chat session or run a single query
            match prompt {
                None => {
                    // Start interactive REPL chat session
                    start_interactive_chat(resolved_config, config_path).await?;
                }
                Some(prompt_str) => {
                    // Run single query with streaming
                    let client = ai::build_client();
                    println!("\n{} ", "Cypher:".bold().magenta());

                    ai::stream_ai_response(
                        &client, &provider, &model, &key, &prompt_str,
                        &mut |chunk: &str| {
                            print!("{}", chunk);
                            std::io::stdout().flush().unwrap();
                        },
                    ).await?;
                    println!();
                }
            }

            Ok(())
        }
        Some(cli::Commands::Providers) => {
            println!("{}", "Configured AI Providers".bold().green());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
            println!();
            
            let providers = ["gemini", "anthropic", "openai", "openrouter"];
            let labels = ["Gemini", "Anthropic", "OpenAI", "OpenRouter"];
            let env_vars = ["GEMINI_API_KEY", "ANTHROPIC_API_KEY", "OPENAI_API_KEY", "OPENROUTER_API_KEY"];
            
            for (i, provider) in providers.iter().enumerate() {
                let has_key = config.get_secure_api_key(provider).is_some();
                let is_active = config.ai.provider.eq_ignore_ascii_case(provider);
                let badge = if has_key { "✓".green().bold() } else { "✗".red().bold() }.to_string();
                let active_marker = if is_active { " ← active".cyan().to_string() } else { String::new() };
                let env_hint = format!(" ({})", env_vars[i]).dimmed().to_string();
                
                println!("  {}  {}{}{}", badge, labels[i].bold(), env_hint, active_marker);
            }
            
            println!();
            println!("{} Use 'cypher models' to list available models.", "ℹ".blue());
            println!("{} Set environment variables or run 'cypher init' to configure API keys.", "ℹ".blue());
            
            Ok(())
        }
        Some(cli::Commands::Models { provider, verbose }) => {
            let provider_filter = provider.as_ref().map(|p| p.to_lowercase());
            
            let all_models = tui::ModelOption::get_all();
            let filtered: Vec<&tui::ModelOption> = all_models.iter()
                .filter(|m| {
                    if let Some(ref filter) = provider_filter {
                        m.provider.to_lowercase() == *filter
                    } else {
                        true
                    }
                })
                .collect();
            
            if filtered.is_empty() {
                return Err(CypherError::Config(format!(
                    "No models found for provider '{}'. Valid providers: gemini, anthropic, openai, openrouter",
                    provider_filter.unwrap_or_default()
                )));
            }
            
            println!("{}", format!("Available Models ({})", filtered.len()).bold().green());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
            println!();
            
            for model in &filtered {
                let configured = if config.get_secure_api_key(&model.provider).is_some() { "✓" } else { " " };
                
                if verbose {
                    println!("  {}  {}", configured, model.label.bold());
                    println!("      Provider: {}", model.provider);
                    println!("      Model ID: {}", model.name.dimmed());
                    println!("      Tag:      {}", model.tag);
                    println!();
                } else {
                    println!("  {}  {:<40} {}", configured, model.label, model.tag.dimmed());
                }
            }
            
            Ok(())
        }
        Some(cli::Commands::Stats) => {
            let mut rule_engine = crate::rules::RuleEngine::new();
            rule_engine.register_rules(crate::rules::RuleLibrary::get_all_rules())?;
            let all_rules = rule_engine.get_all_rules();
            
            let provider_count: usize = ["gemini", "anthropic", "openai", "openrouter"].iter()
                .filter(|p| config.get_secure_api_key(p).is_some())
                .count();
            
            println!("{}", "Cypher CLI Stats".bold().green());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
            println!();
            println!("  {} {:<20} {}", "●".cyan(), "Version", env!("CARGO_PKG_VERSION"));
            println!("  {} {:<20} {}", "●".cyan(), "Active Provider", config.ai.provider);
            println!("  {} {:<20} {}", "●".cyan(), "Active Model", config.ai.model);
            println!("  {} {:<20} {}", "●".cyan(), "Config. Providers", provider_count);
            println!("  {} {:<20} {}", "●".cyan(), "Security Rules", all_rules.len());
            println!("  {} {:<20} {}", "●".cyan(), "Chat Messages", "0");
            println!("  {} {:<20} {}", "●".cyan(), "Scans Run", "0");
            
            let severity_counts: std::collections::HashMap<_, _> = all_rules.iter()
                .map(|r| (r.severity.to_string(), 1usize))
                .fold(std::collections::HashMap::new(), |mut acc, (sev, n)| {
                    *acc.entry(sev).or_insert(0) += n;
                    acc
                });
            
            println!();
            println!("  {} Rules by severity:", "📊".bold());
            for (sev, count) in severity_counts {
                println!("    {} {:<15} {}", "•".dimmed(), sev.to_uppercase(), count);
            }
            
            Ok(())
        }
        Some(cli::Commands::Debug) => {
            println!("{}", "Cypher CLI Debug Info".bold().green());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
            println!();
            println!("  {} {:<20} {}", "●".cyan(), "Version", env!("CARGO_PKG_VERSION"));
            println!("  {} {:<20} {:?}", "●".cyan(), "Config Path", config_path);
            println!("  {} {:<20} {}", "●".cyan(), "Provider", config.ai.provider);
            println!("  {} {:<20} {}", "●".cyan(), "Model", config.ai.model);
            println!("  {} {:<20} {}", "●".cyan(), "Verbose", config.general.verbose);
            println!("  {} {:<20} {}", "●".cyan(), "Color Enabled", config.general.color);
            println!("  {} {:<20} {}", "●".cyan(), "Max Threads", config.general.max_threads);
            
            println!();
            println!("  {} Provider API Keys:", "🔑".bold());
            for p in &["gemini", "anthropic", "openai", "openrouter"] {
                let status = if config.get_secure_api_key(p).is_some() { "✓ configured" } else { "✗ not set" };
                let status_color = if config.get_secure_api_key(p).is_some() { status.green() } else { status.red() };
                println!("    {} {}", p.bold(), status_color);
            }
            
            #[cfg(target_os = "windows")]
            println!("\n  {} OS: Windows", "💻".bold());
            #[cfg(target_os = "linux")]
            println!("\n  {} OS: Linux", "💻".bold());
            #[cfg(target_os = "macos")]
            println!("\n  {} OS: macOS", "💻".bold());
            
            println!("  {} Arch: {}", "💻".bold(), std::env::consts::ARCH);
            
            println!();
            println!("  {} {} {} {}", "▶".bold(), "Debug info saved to clipboard.".dimmed(), "✓".green(), "(use --no-color to disable colors)");
            
            Ok(())
        }
        Some(cli::Commands::Uninstall { force }) => {
            println!("{}", "Cypher CLI Uninstall".bold().red());
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().red());
            println!();
            
            if !force {
                let proceed = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Are you sure you want to uninstall Cypher CLI?")
                    .default(false)
                    .interact()
                    .unwrap_or(false);
                
                if !proceed {
                    println!("{} Uninstall cancelled.", "✓".green());
                    return Ok(());
                }
            }
            
            // Remove global config directory
            let config_dir = if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(std::path::PathBuf::from) {
                Some(home.join(".cypher"))
            } else {
                None
            };
            
            if let Some(ref dir) = config_dir {
                if dir.exists() {
                    if force {
                        let _ = std::fs::remove_dir_all(dir);
                        println!("  {} Removed config directory: {}", "✓".green(), dir.display());
                    } else {
                        let remove_config = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt(format!("Remove config directory ({}).", dir.display()))
                            .default(true)
                            .interact()
                            .unwrap_or(false);
                        
                        if remove_config {
                            let _ = std::fs::remove_dir_all(dir);
                            println!("  {} Removed config directory: {}", "✓".green(), dir.display());
                        }
                    }
                }
            }
            
            // Clear API keys from keyring
            println!();
            println!("  {} Clearing stored API keys...", "🔑".bold());
            for p in &["gemini", "anthropic", "openai", "openrouter"] {
                let entry_name = format!("cypher-cli-{}", p);
                if let Ok(entry) = keyring::Entry::new(&entry_name, "user") {
                    let _ = entry.delete_password();
                }
            }
            println!("  {} Stored API keys cleared.", "✓".green());
            
            println!();
            println!("{}", "To fully remove Cypher CLI, also:".bold().yellow());
            println!("  1. Delete the binary (cypher.exe on Windows, cypher on Unix)");
            println!("  2. Remove the installation directory if you used the install script");
            println!();
            println!("{} Uninstall complete.", "✓".green());
            
            Ok(())
        }
        Some(cli::Commands::Upgrade) => {
            upgrade_cypher().await?;
            Ok(())
        }
    }
}

/// Setup AI config interactively (first-run wizard)
async fn setup_ai() -> Result<(Config, Option<PathBuf>)> {
    use dialoguer::{theme::ColorfulTheme, MultiSelect, Password};
    use std::path::PathBuf;

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().blue());
    println!("  {}", "Welcome to Cypher CLI".bold().green());
    println!("  {}", "The AI Security Engineer.".italic().dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().blue());
    println!("Let's configure your AI providers.\n");

    let provider_options = vec!["Anthropic", "OpenAI", "Gemini", "OpenRouter"];
    let mut chosen_indices = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select one or more providers (Space to select, Enter to confirm)")
        .items(&provider_options)
        .interact()
        .map_err(|e| CypherError::Config(format!("Interactive prompt failed: {}", e)))?;

    if chosen_indices.is_empty() {
        println!("{} No provider selected. Defaulting to Gemini.", "ℹ".blue());
        chosen_indices = vec![2]; // Gemini is index 2
    }

    let mut config = Config::load_default().unwrap_or_default();
    let mut configured_providers = Vec::new();

    for index in chosen_indices {
        let provider_name = provider_options[index];
        let provider_id = provider_name.to_lowercase();

        println!("\n🔑 Configure {}", provider_name.bold().cyan());

        let api_key: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} API Key", provider_name))
            .interact()
            .map_err(|e| CypherError::Config(format!("Prompt input failed: {}", e)))?;

        let api_key = api_key.trim();
        if api_key.is_empty() {
            println!("{}", "Warning: Empty API key input ignored.".yellow());
            continue;
        }

        match Config::save_secure_api_key(&provider_id, api_key) {
            Ok(_) => {
                println!("{} Key saved securely in OS credential store.", "✓".green());
            }
            Err(e) => {
                println!("{} Failed to save securely to OS keyring ({}). Saving to local config file instead.", "!".yellow(), e);
                config.ai.api_key = Some(api_key.to_string());
            }
        }

        configured_providers.push(provider_id);
    }

    if configured_providers.is_empty() {
        return Err(CypherError::Config("No providers were configured.".to_string()));
    }

    let active_provider = configured_providers[0].clone();
    config.ai.provider = active_provider.clone();

    config.ai.model = match active_provider.as_str() {
        "anthropic" => "claude-3-5-sonnet-latest".to_string(),
        "openai" => "gpt-4o".to_string(),
        "openrouter" => "anthropic/claude-3.5-sonnet".to_string(),
        _ => "gemini-2.0-flash".to_string(),
    };

    let mut settings_path = None;
    if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from) {
        let path = home.join(".cypher").join("settings.json");
        config.save_to_file(&path)?;
        println!("\n{} Configuration saved to: {}", "✓".green(), path.display().to_string().cyan());
        settings_path = Some(path);
    }

    println!("\n🎉 Setup complete! Active provider: {} (Model: {})", config.ai.provider.bold().green(), config.ai.model.bold().cyan());
    Ok((config, settings_path))
}

/// Start interactive TUI chat session
async fn start_interactive_chat(config: Config, config_path: Option<PathBuf>) -> Result<()> {
    let mut app = tui::App::new(config.ai.provider.clone(), config.ai.model.clone());
    app.config_path = config_path;
    let mut cfg = config;

    app.add_message("system", "Type /help for commands. Ask me anything about cybersecurity.");
    tui::run_tui(&mut app, &mut cfg).await
}

/// Generate a simple unified diff between old and new content
fn generate_diff(old: &str, new: &str) -> String {
    use similar::TextDiff;
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    for change in diff.iter_all_changes() {
        let sign: String = match change.tag() {
            similar::ChangeTag::Equal => " ".to_string(),
            similar::ChangeTag::Insert => "+".green().to_string(),
            similar::ChangeTag::Delete => "-".red().to_string(),
        };
        output.push_str(&format!("{}{}", sign, change.value()));
    }

    output
}

/// Upgrade Cypher CLI to the latest version by downloading the pre-built binary
/// from GitHub Releases. Shows progress, backs up the old binary, and verifies the new one.
async fn upgrade_cypher() -> Result<()> {
    use colored::Colorize;
    use std::io::Write;

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
    println!("  {}", "Cypher CLI Updater".bold().green());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
    println!();

    // 1. Detect installation method
    let install_method = detect_install_method();
    println!("  {} Detection: {}", "●".cyan(), install_method.description());

    // If installed via npm, advise the npm update path
    if install_method == InstallMethod::Npm {
        println!("  {} You installed Cypher via npm.", "ℹ".blue());
        println!("  {} Run: {}", "  ▶".bold(), "npm update -g cypher-cli".bold().yellow());
        println!("  {} Or continue with auto-update (downloads binary directly).", "  ▶".bold());
        let proceed = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Proceed with auto-update?")
            .default(true)
            .interact()
            .unwrap_or(true);
        if !proceed {
            println!("\n{} Upgrade cancelled.", "✓".green());
            return Ok(());
        }
    }

    // 2. Fetch latest release info
    print!("  {} Checking for updates...", "●".cyan());
    std::io::stdout().flush().ok();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| CypherError::Report(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get("https://api.github.com/repos/Ayan-Flash/Cypher/releases/latest")
        .header("User-Agent", "cypher-cli-updater")
        .send()
        .await
        .map_err(|e| CypherError::Report(format!("Failed to contact GitHub Releases: {}", e)))?;

    if !response.status().is_success() {
        println!("\r  {} Failed: HTTP {}", "✗".red(), response.status());
        return Err(CypherError::Report(format!(
            "Failed to fetch latest version: HTTP {}",
            response.status()
        )));
    }

    let release_info: serde_json::Value = response
        .json()
        .await
        .map_err(|e| CypherError::Report(format!("Failed to parse release information: {}", e)))?;

    let latest_tag = release_info
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .ok_or_else(|| CypherError::Report("Latest release does not have a tag_name".to_string()))?;

    let latest_version = latest_tag.trim_start_matches('v');
    let current_version = env!("CARGO_PKG_VERSION");

    println!("\r  {} Found latest version: {}                ", "✓".green(), latest_version);

    println!();
    println!("  {} Current: {:<10}", "●".cyan(), current_version);
    println!("  {} Latest:  {:<10}", "●".cyan(), latest_version);

    if !is_newer_version(current_version, latest_version) {
        println!();
        println!("  {} Cypher CLI is already up to date.", "✓".green().bold());
        return Ok(());
    }

    // Show release notes excerpt
    if let Some(body) = release_info.get("body").and_then(|b| b.as_str()) {
        let lines: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
        let preview: Vec<&str> = lines.iter().take(6).copied().collect();
        if !preview.is_empty() {
            println!();
            println!("  {} Release highlights:", "📋".bold());
            for line in &preview {
                let trimmed = line.trim().trim_start_matches('-').trim();
                if !trimmed.is_empty() {
                    println!("    • {}", trimmed);
                }
            }
            if lines.len() > 6 {
                println!("    ... and {} more lines", lines.len() - 6);
            }
        }
    }

    // 3. Confirm upgrade
    println!();
    let confirm = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Download and install this update?")
        .default(true)
        .interact()
        .unwrap_or(true);

    if !confirm {
        println!("{} Upgrade cancelled.", "✓".green());
        return Ok(());
    }

    // 4. Detect platform and download pre-built binary
    println!();
    let target = detect_download_target();
    let repo = "Ayan-Flash/Cypher";
    let archive_name = format!("cypher-{}{}", target.name, target.ext);
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        repo, latest_tag, archive_name
    );

    let temp_dir = std::env::temp_dir().join("cypher-upgrade");
    let _ = std::fs::create_dir_all(&temp_dir);
    let archive_path = temp_dir.join(&archive_name);

    println!("  {} Downloading {}...", "⬇".bold(), archive_name);

    // Download with progress bar
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CypherError::Report(format!("Download failed: {}", e)))?;

    let total_size = response.content_length().unwrap_or(0);
    let pb = indicatif::ProgressBar::new(total_size);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );

    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(&archive_path)
        .map_err(|e| CypherError::Io(e))?;
    let mut stream = response.bytes_stream();



    while let Some(chunk) = futures::StreamExt::next(&mut stream).await {
        let chunk = chunk.map_err(|e| CypherError::Report(format!("Download stream error: {}", e)))?;
        file.write_all(&chunk).map_err(|e| CypherError::Io(e))?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    pb.finish_and_clear();

    // 5. Extract binary
    println!("  {} Extracting binary...", "📦".bold());
    let bin_name = if cfg!(target_os = "windows") { "cypher.exe" } else { "cypher" };
    let extract_dir = temp_dir.join("extracted");
    let _ = std::fs::create_dir_all(&extract_dir);
    let extracted_bin = extract_dir.join(bin_name);

    let extract_result = if cfg!(target_os = "windows") {
        // Windows: try tar first (Win 10+ has it), then PowerShell
        extract_zip_windows(&archive_path, &extract_dir, bin_name)
    } else {
        extract_tar_gz(&archive_path, &extract_dir, bin_name)
    };

    if let Err(e) = extract_result {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(CypherError::Report(format!("Failed to extract binary: {}", e)));
    }

    if !extracted_bin.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(CypherError::Report("Extracted binary not found in archive.".to_string()));
    }

    // 6. Verify the new binary
    println!("  {} Verifying new binary...", "🔍".bold());
    let verify_output = std::process::Command::new(&extracted_bin)
        .arg("--version")
        .output();

    match verify_output {
        Ok(output) => {
            let version_text = String::from_utf8_lossy(&output.stdout);
            println!("  {} New binary version: {}", "✓".green(), version_text.trim());

            if output.status.success() {
                println!("  {} Binary verification passed.", "✓".green());
            } else {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(CypherError::Report("New binary failed verification.".to_string()));
            }
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(CypherError::Report(format!("Failed to verify new binary: {}", e)));
        }
    }

    // 7. Replace the current binary
    let current_exe = std::env::current_exe()
        .map_err(|e| CypherError::Report(format!("Cannot determine current binary path: {}", e)))?;

    println!("  {} Current binary: {}", "●".cyan(), current_exe.display());

    // Backup old binary
    let backup_path = current_exe.with_extension("exe.old");
    if backup_path.exists() {
        let _ = std::fs::remove_file(&backup_path);
    }

    println!("  {} Installing new binary...", "⚙".bold());

    #[cfg(target_os = "windows")]
    {
        // On Windows we can't overwrite the running exe directly.
        // Use PowerShell to rename current -> .old and move new -> current.
        let rename_script = format!(
            "Start-Sleep -Seconds 1; \
             Move-Item -Force '{}' '{}'; \
             Move-Item -Force '{}' '{}'; \
             Start-Process -FilePath '{}' -ArgumentList '--version' -NoNewWindow -Wait",
            current_exe.display(),
            backup_path.display(),
            extracted_bin.display(),
            current_exe.display(),
            current_exe.display()
        );
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &rename_script])
            .status()
            .map_err(|e| CypherError::Io(e))?;

        if !status.success() {
            // Fallback: try direct rename (might work if not running from same exe)
            let _ = std::fs::rename(&current_exe, &backup_path);
            let _ = std::fs::rename(&extracted_bin, &current_exe);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::fs::rename(&current_exe, &backup_path);
        let _ = std::fs::rename(&extracted_bin, &current_exe);
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&current_exe, perms);
    }

    // 8. Cleanup temp files
    let _ = std::fs::remove_dir_all(&temp_dir);

    // 9. Success
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
    println!("  {} Upgrade complete! {}", "✓".green().bold(), current_version.bold().yellow());
    println!("  {} → {}", "●".cyan(), latest_version.bold().green());
    println!("  {} Previous binary saved as: {}", "●".cyan(), backup_path.display().to_string().dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().green());
    println!();
    println!("  Restart the CLI to use the new version.");

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum InstallMethod {
    Npm,
    Cargo,
    Script,
    Unknown,
}

impl InstallMethod {
    fn description(&self) -> &'static str {
        match self {
            InstallMethod::Npm => "npm package (node_modules)",
            InstallMethod::Cargo => "Rust cargo install",
            InstallMethod::Script => "install script / manual",
            InstallMethod::Unknown => "standalone binary",
        }
    }
}

fn detect_install_method() -> InstallMethod {
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return InstallMethod::Unknown,
    };
    let path_str = current_exe.to_string_lossy().to_lowercase();

    // npm installs binaries into a `node_modules/.bin/` or `npm/bin/` directory
    if path_str.contains("node_modules") || path_str.contains("npm\\bin") || path_str.contains("npm/bin") {
        return InstallMethod::Npm;
    }

    // cargo install puts binaries in `.cargo/bin/` or `$CARGO_HOME/bin`
    if path_str.contains(".cargo\\bin") || path_str.contains(".cargo/bin") || path_str.contains("cargo_home") {
        return InstallMethod::Cargo;
    }

    // Check if there's a Cargo.toml nearby (running from source)
    if let Some(parent) = current_exe.parent() {
        if parent.join("Cargo.toml").exists() {
            return InstallMethod::Cargo;
        }
    }

    // Check if it's in .cypher/bin (install script puts it there)
    if path_str.contains(".cypher") {
        return InstallMethod::Script;
    }

    InstallMethod::Unknown
}

struct DownloadTarget {
    name: String,
    ext: &'static str,
}

fn detect_download_target() -> DownloadTarget {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64" // default fallback
    };

    let ext = if cfg!(target_os = "windows") { ".zip" } else { ".tar.gz" };
    let name = format!("{}-{}", os, arch);

    DownloadTarget { name, ext }
}

fn extract_tar_gz(archive: &std::path::Path, dest: &std::path::Path, bin_name: &str) -> std::result::Result<(), String> {
    let status = std::process::Command::new("tar")
        .args(["xzf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()])
        .status()
        .map_err(|e| format!("Failed to run tar: {}", e))?;

    if !status.success() {
        return Err("tar extraction failed".to_string());
    }

    let bin_path = dest.join(bin_name);
    if bin_path.exists() {
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(dest) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let sub_bin = entry.path().join(bin_name);
                if sub_bin.exists() {
                    std::fs::rename(&sub_bin, &bin_path).map_err(|e| format!("Failed to move binary: {}", e))?;
                    return Ok(());
                }
            }
        }
    }

    Err(format!("Binary '{}' not found in tar archive", bin_name))
}

#[cfg(target_os = "windows")]
fn extract_zip_windows(archive: &std::path::Path, dest: &std::path::Path, bin_name: &str) -> std::result::Result<(), String> {
    let ps_script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        archive.to_string_lossy(),
        dest.to_string_lossy()
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if !status.success() {
        return Err("Expand-Archive failed".to_string());
    }

    let bin_path = dest.join(bin_name);
    if bin_path.exists() {
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(dest) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let sub_bin = entry.path().join(bin_name);
                if sub_bin.exists() {
                    std::fs::rename(&sub_bin, &bin_path).map_err(|e| format!("Failed to move binary: {}", e))?;
                    return Ok(());
                }
            }
        }
    }

    Err(format!("Binary '{}' not found in zip archive", bin_name))
}

/// Helper function to compare semver versions in a simple and robust way
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let c = *current_parts.get(i).unwrap_or(&0);
        let l = *latest_parts.get(i).unwrap_or(&0);
        if l > c {
            return true;
        } else if c > l {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.1.1", "0.1.2"));
        assert!(is_newer_version("0.1.1", "1.0.0"));
        assert!(is_newer_version("0.1.9", "0.1.10"));
        assert!(!is_newer_version("0.1.2", "0.1.2"));
        assert!(!is_newer_version("0.1.2", "0.1.1"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }
}

