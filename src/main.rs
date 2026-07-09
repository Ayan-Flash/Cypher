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

use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

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
    let config = if let Some(config_path) = &cli.config {
        Config::load_from_file(config_path)?
    } else {
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
                let resolved_config = setup_ai().await?;
                start_interactive_chat(resolved_config).await?;
            } else {
                start_interactive_chat(config).await?;
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

            let client = reqwest::Client::new();
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
                        println!("\n  {} Failed to generate fix: {:?}", "✗".red(), e);
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
                    start_interactive_chat(resolved_config).await?;
                }
                Some(prompt_str) => {
                    // Run single query with streaming
                    let client = reqwest::Client::new();
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
    }
}

/// Setup AI config interactively (first-run wizard)
async fn setup_ai() -> Result<Config> {
    use dialoguer::{theme::ColorfulTheme, MultiSelect, Password};
    use std::path::PathBuf;

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().blue());
    println!("  {}", "Welcome to Cypher CLI".bold().green());
    println!("  {}", "The AI Security Engineer.".italic().dimmed());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bold().blue());
    println!("Let's configure your AI providers.\n");

    let provider_options = vec!["Anthropic", "OpenAI", "Gemini", "OpenRouter"];
    let chosen_indices = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select one or more providers (Space to select, Enter to confirm)")
        .items(&provider_options)
        .interact()
        .map_err(|e| CypherError::Config(format!("Interactive prompt failed: {}", e)))?;

    if chosen_indices.is_empty() {
        return Err(CypherError::Config("At least one provider must be selected.".to_string()));
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
        "anthropic" => "claude-sonnet-5".to_string(),
        "openai" => "gpt-5.5".to_string(),
        "openrouter" => "anthropic/claude-opus-4-8".to_string(),
        _ => "gemini-3.5-flash".to_string(),
    };

    if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from) {
        let settings_path = home.join(".cypher").join("settings.json");
        config.save_to_file(&settings_path)?;
        println!("\n{} Configuration saved to: {}", "✓".green(), settings_path.display().to_string().cyan());
    }

    println!("\n🎉 Setup complete! Active provider: {} (Model: {})", config.ai.provider.bold().green(), config.ai.model.bold().cyan());
    Ok(config)
}

/// Start interactive TUI chat session
async fn start_interactive_chat(config: Config) -> Result<()> {
    let mut app = tui::App::new(config.ai.provider.clone(), config.ai.model.clone());
    let mut cfg = config;

    app.add_message("system", "Type \\help for commands. Ask me anything about cybersecurity.");
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

