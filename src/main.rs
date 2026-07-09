mod cli;
mod config;
mod error;
mod rules;
mod parser;
mod reporter;
mod detector;
mod scanner;

use cli::Cli;
use config::Config;
use error::{Result, CypherError};
use rules::{RuleEngine, RuleLibrary, Severity, RuleCategory};
use scanner::Scanner;
use colored::Colorize;

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
                .map_err(|e| CypherError::InvalidSeverity(e))?;
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
            println!("\n🔍 Fix Mode implementation in progress... (Dry run: {})", dry_run);
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
                    .map_err(|e| CypherError::InvalidSeverity(e))?;
                rules.retain(|r| r.severity == target_severity);
            }

            if let Some(cat_str) = category {
                let target_category: RuleCategory = std::str::FromStr::from_str(&cat_str)
                    .map_err(|e| CypherError::Rule(e))?;
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
                    // Run single query
                    println!("Sending query to Cypher AI Assistant (Provider: {}, Model: {})...", provider, model);

                    let client = reqwest::Client::new();
                    let response = call_ai_api(&client, &provider, &model, &key, &prompt_str).await?;

                    println!("\n=== Cypher AI Assistant ===");
                    println!("{}", response);
                    println!("=============================");
                }
            }

            Ok(())
        }
    }
}

/// Setup AI config interactively
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

        // Save securely using keyring
        match Config::save_secure_api_key(&provider_id, api_key) {
            Ok(_) => {
                println!("{} Key saved securely in OS credential store.", "✓".green());
            }
            Err(e) => {
                println!("{} Failed to save securely to OS keyring ({}). Saving to local config file instead.", "!".yellow(), e);
                // Fallback to local config file
                config.ai.api_key = Some(api_key.to_string());
            }
        }
        
        configured_providers.push(provider_id);
    }

    if configured_providers.is_empty() {
        return Err(CypherError::Config("No providers were configured.".to_string()));
    }

    // Set active provider (default to the first one configured)
    let active_provider = configured_providers[0].clone();
    config.ai.provider = active_provider.clone();
    
    // Set default model for active provider
    config.ai.model = match active_provider.as_str() {
        "anthropic" => "claude-3-5-sonnet-20240620".to_string(),
        "openai" => "gpt-4o".to_string(),
        "openrouter" => "meta-llama/llama-3.1-70b-instruct".to_string(),
        _ => "gemini-1.5-flash".to_string(),
    };

    // Save configuration settings
    if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from) {
        let settings_path = home.join(".cypher").join("settings.json");
        config.save_to_file(&settings_path)?;
        println!("\n{} Configuration saved to: {}", "✓".green(), settings_path.display().to_string().cyan());
    }

    println!("\n🎉 Setup complete! Active provider: {} (Model: {})", config.ai.provider.bold().green(), config.ai.model.bold().cyan());
    Ok(config)
}

/// Start interactive REPL chat session
async fn start_interactive_chat(mut config: Config) -> Result<()> {
    use std::io::Write;
    use std::path::PathBuf;

    let banner = r#"
   ______            __               
  / ____/_  ______  / /_  ___  _____ 
 / /   / / / / __ \/ __ \/ _ \/ ___/ 
/ /___/ /_/ / /_/ / / / /  __/ /     
\____/\__, / .___/_/ /_/\___/_/      
     /____/_/                        
"#;
    println!("{}", banner.bold().magenta());
    println!("  🛡️  {}", "Security-First AI Assistant v0.1.0".bold().dimmed());
    println!();
    println!("  {} Provider:  {}", "🧠".blue(), config.ai.provider.bold().cyan());
    println!("  {} Model:     {}", "🤖".blue(), config.ai.model.bold().cyan());
    println!("  {} Path:      {}", "📁".blue(), std::env::current_dir().unwrap_or_default().display().to_string().yellow());
    println!();
    println!("  Type your security queries or use commands:");
    println!("    {}  Display active commands", "\\help".bold().cyan());
    println!("    {}   Scan local directory for bugs", "\\scan".bold().cyan());
    println!("    {}   Change AI model or provider", "\\models".bold().cyan());
    println!("    {}   Quit session", "\\exit".bold().cyan());
    println!();

    let client = reqwest::Client::new();

    loop {
        let model_display = config.ai.model.cyan();
        print!("\n{} {}{} {} ", "cypher".bold().magenta(), "[".dimmed(), model_display, "] ›".bold().green());
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") || input == "\\exit" || input == "\\quit" {
            println!("Goodbye!");
            break;
        }

        // Handle slash commands
        if input.starts_with('\\') {
            let cmd = input.to_lowercase();
            if cmd == "\\help" {
                println!("\n{}", "🛡️ Cypher CLI Chat Commands:".bold().green());
                println!("  \\models - List available models and switch AI provider");
                println!("  \\scan   - Scan the current directory for security issues");
                println!("  \\help   - Display this help message");
                println!("  \\exit   - Exit the interactive session");
            } else if cmd == "\\scan" {
                println!("\n{}", "🔍 Starting security scan of the current directory...".bold().cyan());
                let scan_path = std::env::current_dir().unwrap();
                
                let mut detector = detector::Detector::new();
                if let Ok(frameworks) = detector.detect(&scan_path) {
                    if !frameworks.is_empty() {
                        println!("Detected frameworks: {}", frameworks.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", "));
                    }
                }
                
                let mut rule_engine = RuleEngine::new();
                let _ = rule_engine.register_rules(RuleLibrary::get_all_rules());
                let scanner = Scanner::new(config.clone(), rule_engine);
                
                match scanner.scan(&scan_path).await {
                    Ok(results) => {
                        let reporter = reporter::create_reporter("text").unwrap();
                        let report_content = reporter.generate(&results).unwrap();
                        println!("{}", report_content);
                    }
                    Err(e) => {
                        println!("{} Scan failed: {:?}", "Error:".red(), e);
                    }
                }
            } else if cmd == "\\models" {
                let providers = vec!["anthropic", "openai", "gemini", "openrouter"];
                let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Select AI Provider")
                    .items(&providers)
                    .default(0)
                    .interact()
                    .map_err(|e| CypherError::Config(format!("Prompt selection failed: {}", e)))?;
                    
                let provider = providers[selection];
                
                let models = match provider {
                    "anthropic" => vec!["claude-3-5-sonnet-20240620", "claude-3-opus-20240229", "claude-3-haiku-20240307"],
                    "openai" => vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
                    "openrouter" => vec![
                        "meta-llama/llama-3.1-70b-instruct",
                        "anthropic/claude-3.5-sonnet",
                        "google/gemini-flash-1.5",
                        "openai/gpt-4o-mini",
                    ],
                    _ => vec!["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash-exp"],
                };
                
                let model_selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Select Model")
                    .items(&models)
                    .default(0)
                    .interact()
                    .map_err(|e| CypherError::Config(format!("Prompt selection failed: {}", e)))?;
                    
                let model = models[model_selection].to_string();
                
                let has_key = config.get_secure_api_key(provider).is_some();
                if !has_key {
                    println!("{}", format!("⚠️ No API key configured for provider '{}'.", provider).yellow());
                    let setup_now = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Would you like to enter the API key now?")
                        .default(true)
                        .interact()
                        .unwrap_or(false);
                        
                    if setup_now {
                        let api_key = dialoguer::Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt(format!("{} API Key", provider))
                            .interact()
                            .map_err(|e| CypherError::Config(format!("Input failed: {}", e)))?;
                            
                        let api_key = api_key.trim();
                        if !api_key.is_empty() {
                            if let Err(e) = Config::save_secure_api_key(provider, api_key) {
                                println!("{} Failed to save securely ({}). Saving to local config file instead.", "!".yellow(), e);
                                config.ai.api_key = Some(api_key.to_string());
                            } else {
                                println!("{} API key saved securely.", "✓".green());
                                config.ai.api_key = None;
                            }
                        }
                    }
                }
                
                config.ai.provider = provider.to_string();
                config.ai.model = model;
                
                if let Some(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok().map(PathBuf::from) {
                    let settings_path = home.join(".cypher").join("settings.json");
                    let _ = config.save_to_file(&settings_path);
                }
                
                println!("\nActive provider switched to {} ({})", config.ai.provider.bold().green(), config.ai.model.bold().cyan());
            } else {
                println!("Unknown command. Type '\\help' to see available commands.");
            }
            continue;
        }

        // Get active API key dynamically (supports keyring changes)
        let provider = &config.ai.provider;
        let model = &config.ai.model;
        let api_key = config.get_secure_api_key(provider).unwrap_or_default();

        if api_key.is_empty() {
            println!("{} No API key found for provider '{}'. Please use '\\models' to switch or configure it.", "Error:".red().bold(), provider);
            continue;
        }

        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_style(
            indicatif::ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.magenta} {msg:.dim}")
                .unwrap(),
        );
        spinner.set_message("Analyzing request...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        // Call AI API
        let response_result = call_ai_api(&client, provider, model, &api_key, input).await;
        
        spinner.finish_and_clear();

        match response_result {
            Ok(response) => {
                println!("\n{}", response);
            }
            Err(e) => {
                println!("{} {:?}", "Error:".red().bold(), e);
            }
        }
    }

    Ok(())
}

/// Call correct AI provider API
async fn call_ai_api(
    client: &reqwest::Client,
    provider: &str,
    model: &str,
    api_key: &str,
    prompt: &str,
) -> Result<String> {
    let system_prompt = "You are the Cypher Security AI Assistant. You are a world-class cybersecurity expert and secure developer. \
                         You must focus exclusively on software security, web security, cryptography, DevSecOps, vulnerabilities, secure coding, and threat modeling. \
                         If the user asks a general programming, architectural, or technical question, do NOT refuse it outright. Instead, you must analyze and answer it specifically from a security angle \
                         (e.g., discuss security implications, inputs/bounds validation, threat scenarios, resource exhaustion, or secure coding best practices related to their question). \
                         Only if the user's request is completely non-technical, non-software-related, or cannot possibly be framed from a security perspective should you politely decline \
                         and direct them back to cybersecurity topics.";

    match provider {
        "anthropic" => {
            let url = "https://api.anthropic.com/v1/messages";
            let request_body = serde_json::json!({
                "model": model,
                "max_tokens": 1024,
                "system": system_prompt,
                "messages": [
                    {"role": "user", "content": prompt}
                ]
            });

            let response = client.post(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to connect to Anthropic API: {}", e)))?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(CypherError::Scanner(format!("Anthropic API returned error: {}", err_text)));
            }

            let response_json: serde_json::Value = response.json()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to parse Anthropic response: {}", e)))?;

            if let Some(content) = response_json.get("content").and_then(|c| c.as_array()) {
                if let Some(first_part) = content.get(0) {
                    if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                        return Ok(text.to_string());
                    }
                }
            }
            Err(CypherError::Scanner("Invalid response structure from Anthropic API".to_string()))
        }
        "openrouter" => {
            let url = "https://openrouter.ai/api/v1/chat/completions";
            let request_body = serde_json::json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": prompt}
                ]
            });

            let response = client.post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to connect to OpenRouter API: {}", e)))?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(CypherError::Scanner(format!("OpenRouter API returned error: {}", err_text)));
            }

            let response_json: serde_json::Value = response.json()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to parse OpenRouter response: {}", e)))?;

            if let Some(choices) = response_json.get("choices").and_then(|c| c.as_array()) {
                if let Some(first_choice) = choices.get(0) {
                    if let Some(message) = first_choice.get("message") {
                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                            return Ok(content.to_string());
                        }
                    }
                }
            }
            Err(CypherError::Scanner("Invalid response structure from OpenRouter API".to_string()))
        }
        _ => {
            // Default to Gemini
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, api_key
            );

            let request_body = serde_json::json!({
                "contents": [{
                    "parts": [{
                        "text": format!("System Instruction: {}\n\nUser Question: {}", system_prompt, prompt)
                    }]
                }]
            });

            let response = client.post(&url)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to connect to Gemini API: {}", e)))?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(CypherError::Scanner(format!("Gemini API returned error: {}", err_text)));
            }

            let response_json: serde_json::Value = response.json()
                .await
                .map_err(|e| CypherError::Scanner(format!("Failed to parse Gemini response: {}", e)))?;

            if let Some(candidates) = response_json.get("candidates") {
                if let Some(first_candidate) = candidates.get(0) {
                    if let Some(content) = first_candidate.get("content") {
                        if let Some(parts) = content.get("parts") {
                            if let Some(first_part) = parts.get(0) {
                                if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                                    return Ok(text.to_string());
                                }
                            }
                        }
                    }
                }
            }
            Err(CypherError::Scanner("Invalid response structure from Gemini API".to_string()))
        }
    }
}


