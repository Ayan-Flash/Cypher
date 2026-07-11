use crate::ai;
use crate::config::Config;
use crate::error::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::io;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    SelectModelDialog {
        search_query: String,
        selected_index: usize,
    },
    SelectProviderDialog {
        selected_index: usize,
    },
    ApiKeyEntry {
        provider: String,
        key_input: String,
        cursor_position: usize,
    },
    CommandPalette {
        search_query: String,
        selected_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
}

impl CommandOption {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self { name: "/clear".to_string(), description: "Clear the conversation".to_string() },
            Self { name: "/copy".to_string(), description: "Copy last response, code, or chat transcript".to_string() },
            Self { name: "/exit".to_string(), description: "Exit the app".to_string() },
            Self { name: "/export".to_string(), description: "Export conversation to markdown file".to_string() },
            Self { name: "/help".to_string(), description: "Help".to_string() },
            Self { name: "/models".to_string(), description: "Switch provider and model".to_string() },
            Self { name: "/retry".to_string(), description: "Retry the last message".to_string() },
            Self { name: "/scan".to_string(), description: "Scan current directory".to_string() },
            Self { name: "/upgrade".to_string(), description: "Upgrade Cypher CLI".to_string() },
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOption {
    pub provider: String,
    pub name: String,
    pub label: String,
    pub tag: String,
}

impl ModelOption {
    /// The full catalog of selectable models. Each entry's `provider` is the actual API this
    /// model is served through, and `tag` always names that same provider — never a different
    /// vendor's brand, so the model dialog can't misrepresent which service a request goes to.
    pub fn get_all() -> Vec<Self> {
        vec![
            Self { provider: "anthropic".to_string(), name: "claude-3-5-sonnet-latest".to_string(), label: "Claude 3.5 Sonnet".to_string(), tag: "Anthropic".to_string() },
            Self { provider: "anthropic".to_string(), name: "claude-3-5-haiku-latest".to_string(), label: "Claude 3.5 Haiku".to_string(), tag: "Anthropic".to_string() },
            Self { provider: "anthropic".to_string(), name: "claude-3-opus-latest".to_string(), label: "Claude 3 Opus".to_string(), tag: "Anthropic".to_string() },
            Self { provider: "openai".to_string(), name: "gpt-4o".to_string(), label: "GPT-4o".to_string(), tag: "OpenAI".to_string() },
            Self { provider: "openai".to_string(), name: "gpt-4o-mini".to_string(), label: "GPT-4o Mini".to_string(), tag: "OpenAI".to_string() },
            Self { provider: "openai".to_string(), name: "o1".to_string(), label: "o1".to_string(), tag: "OpenAI".to_string() },
            Self { provider: "openai".to_string(), name: "o1-mini".to_string(), label: "o1-mini".to_string(), tag: "OpenAI".to_string() },
            Self { provider: "gemini".to_string(), name: "gemini-1.5-flash".to_string(), label: "Gemini 1.5 Flash".to_string(), tag: "Gemini".to_string() },
            Self { provider: "gemini".to_string(), name: "gemini-1.5-pro".to_string(), label: "Gemini 1.5 Pro".to_string(), tag: "Gemini".to_string() },
            Self { provider: "gemini".to_string(), name: "gemini-2.0-flash".to_string(), label: "Gemini 2.0 Flash".to_string(), tag: "Gemini".to_string() },
            Self { provider: "openrouter".to_string(), name: "anthropic/claude-3.5-sonnet".to_string(), label: "Claude 3.5 Sonnet (via OpenRouter)".to_string(), tag: "OpenRouter".to_string() },
            Self { provider: "openrouter".to_string(), name: "meta-llama/llama-3.3-70b-instruct".to_string(), label: "Llama 3.3 70B (via OpenRouter)".to_string(), tag: "OpenRouter".to_string() },
            Self { provider: "openrouter".to_string(), name: "deepseek/deepseek-chat".to_string(), label: "DeepSeek Chat (via OpenRouter)".to_string(), tag: "OpenRouter".to_string() },
            Self { provider: "openrouter".to_string(), name: "qwen/qwen-2.5-coder-32b-instruct".to_string(), label: "Qwen 2.5 Coder 32B (via OpenRouter)".to_string(), tag: "OpenRouter".to_string() },
        ]
    }
}

pub enum AiEvent {
    Chunk(String),
    Done,
    Error(String),
}

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub scroll_to_bottom: bool,
    pub provider: String,
    pub model: String,
    pub loading: bool,
    pub exit: bool,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub temp_input: String,
    pub mode: AppMode,
    pub command_menu_index: usize,
    /// Providers that currently have a usable API key (env var, keyring, or config file).
    /// Refreshed whenever the model dialog opens so the indicator can't go stale.
    pub configured_providers: HashSet<String>,
    /// Handle to the in-flight AI request, if any, so Escape can cancel it without killing the app.
    pub current_task: Option<JoinHandle<()>>,
    /// The last prompt sent to the AI, so /retry can resend it.
    pub last_user_prompt: Option<String>,
    /// Path to the loaded configuration file, so settings modifications in TUI save correctly.
    pub config_path: Option<std::path::PathBuf>,
    /// Recently used models (up to 3), shown at top of model dialog.
    pub recent_models: Vec<String>,
}

impl App {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            scroll_to_bottom: true,
            provider,
            model,
            loading: false,
            exit: false,
            history: Vec::new(),
            history_index: None,
            temp_input: String::new(),
            mode: AppMode::Normal,
            command_menu_index: 0,
            configured_providers: HashSet::new(),
            current_task: None,
            last_user_prompt: None,
            config_path: None,
            recent_models: Vec::new(),
        }
    }

    /// Refresh which providers currently have a usable API key configured.
    pub fn refresh_configured_providers(&mut self, config: &Config) {
        self.configured_providers.clear();
        for provider in ["anthropic", "openai", "gemini", "openrouter"] {
            if config.get_secure_api_key(provider).is_some() {
                self.configured_providers.insert(provider.to_string());
            }
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
        self.scroll_to_bottom = true;
    }

    pub fn append_to_last(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            last.content.push_str(content);
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let mut chars: Vec<char> = self.input.chars().collect();
        chars.insert(self.cursor_position, c);
        self.input = chars.into_iter().collect();
        self.cursor_position += 1;
    }

    #[allow(dead_code)]
    pub fn insert_str(&mut self, s: &str) {
        let mut chars: Vec<char> = self.input.chars().collect();
        for (i, c) in s.chars().enumerate() {
            chars.insert(self.cursor_position + i, c);
        }
        self.input = chars.into_iter().collect();
        self.cursor_position += s.chars().count();
    }

    pub fn delete_char(&mut self) {
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_position < chars.len() {
            chars.remove(self.cursor_position);
            self.input = chars.into_iter().collect();
        }
    }

    pub fn backspace_char(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.input.chars().collect();
            chars.remove(self.cursor_position - 1);
            self.input = chars.into_iter().collect();
            self.cursor_position -= 1;
        }
    }

    pub fn get_total_lines(&self, width: u16) -> usize {
        let mut total_lines = 0;
        for msg in &self.messages {
            // Label
            total_lines += 1;
            // Content lines
            for line in msg.content.lines() {
                let available_width = (width as usize).saturating_sub(4).max(10);
                let line_len = line.chars().count();
                let wrapped_lines = (line_len + available_width - 1) / available_width;
                total_lines += wrapped_lines.max(1);
            }
            // Blank separator line
            total_lines += 1;
        }
        if self.loading {
            total_lines += 1;
        }
        total_lines
    }

    pub fn get_autocomplete_commands(&self) -> Vec<CommandOption> {
        let all = CommandOption::get_all();
        if self.input.starts_with('/') {
            let filter = self.input.to_lowercase();
            all.into_iter()
                .filter(|c| c.name.to_lowercase().starts_with(&filter))
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub fn setup_terminal() -> io::Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste
    )?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal() -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableBracketedPaste
    )?;
    Ok(())
}

pub async fn run_tui(app: &mut App, config: &mut Config) -> Result<()> {
    let mut terminal = setup_terminal().map_err(crate::error::CypherError::Io)?;
    let (ai_tx, mut ai_rx) = mpsc::unbounded_channel::<AiEvent>();
    let mut event_stream = EventStream::new();

    let mut api_key = config.get_secure_api_key(&config.ai.provider).unwrap_or_default();
    app.refresh_configured_providers(config);

    // Initial scroll update
    if app.scroll_to_bottom {
        if let Ok(size) = terminal.size() {
            let msg_width = size.width;
            let msg_height = size.height.saturating_sub(4);
            let total_lines = app.get_total_lines(msg_width);
            app.scroll_offset = (total_lines as u16).saturating_sub(msg_height);
        }
    }

    terminal.draw(|f| draw(f, app)).map_err(crate::error::CypherError::Io)?;

    loop {
        if app.exit {
            break;
        }

        tokio::select! {
            Some(Ok(event)) = event_stream.next() => {
                match event {
                    Event::Key(key) => {
                        handle_key(key, app, config, &mut api_key, &ai_tx).await;
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
            Some(event) = ai_rx.recv() => {
                match event {
                    AiEvent::Chunk(text) => {
                        app.append_to_last(&text);
                    }
                    AiEvent::Done => {
                        app.loading = false;
                        app.current_task = None;
                    }
                    AiEvent::Error(e) => {
                        app.loading = false;
                        app.current_task = None;
                        app.add_message("system", &format!("Error: {}", e));
                    }
                }
            }
        }

        if app.scroll_to_bottom {
            if let Ok(size) = terminal.size() {
                let msg_width = size.width;
                let msg_height = size.height.saturating_sub(4);
                let total_lines = app.get_total_lines(msg_width);
                app.scroll_offset = (total_lines as u16).saturating_sub(msg_height);
            }
        }

        let _ = terminal.draw(|f| draw(f, app));
    }

    restore_terminal().map_err(crate::error::CypherError::Io)?;
    Ok(())
}

async fn handle_key(
    key: KeyEvent,
    app: &mut App,
    config: &mut Config,
    api_key: &mut String,
    ai_tx: &mpsc::UnboundedSender<AiEvent>,
) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    // Ctrl+C exits immediately
    if key.modifiers == KeyModifiers::CONTROL && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('C')) {
        app.exit = true;
        return;
    }

    // Escape cancels an in-flight request without exiting the app.
    if key.code == KeyCode::Esc && app.loading && matches!(app.mode, AppMode::Normal) {
        if let Some(handle) = app.current_task.take() {
            handle.abort();
        }
        app.loading = false;
        app.add_message("system", "Cancelled.");
        return;
    }

    if app.loading {
        return;
    }

    match &mut app.mode {
        AppMode::SelectModelDialog { search_query, selected_index } => {
            match key.code {
                KeyCode::Esc => {
                    app.mode = AppMode::Normal;
                }
                KeyCode::Up => {
                    let models = ModelOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered_count = models.iter()
                        .filter(|m| m.label.to_lowercase().contains(&query) || m.provider.to_lowercase().contains(&query))
                        .count();
                    if filtered_count > 0 {
                        *selected_index = if *selected_index == 0 {
                            filtered_count - 1
                        } else {
                            *selected_index - 1
                        };
                    }
                }
                KeyCode::Down => {
                    let models = ModelOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered_count = models.iter()
                        .filter(|m| m.label.to_lowercase().contains(&query) || m.provider.to_lowercase().contains(&query))
                        .count();
                    if filtered_count > 0 {
                        *selected_index = (*selected_index + 1) % filtered_count;
                    }
                }
                KeyCode::Backspace => {
                    search_query.pop();
                    *selected_index = 0;
                }
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL && (c == 'a' || c == 'A') {
                        // Ctrl+A: open provider configuration dialog
                        app.mode = AppMode::SelectProviderDialog {
                            selected_index: 0,
                        };
                        return;
                    }
                    search_query.push(c);
                    *selected_index = 0;
                }
                KeyCode::Enter => {
                    let models = ModelOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered: Vec<ModelOption> = models.into_iter()
                        .filter(|m| m.label.to_lowercase().contains(&query) || m.provider.to_lowercase().contains(&query))
                        .collect();

                    if !filtered.is_empty() {
                        let selected = &filtered[*selected_index % filtered.len()];
                        config.ai.provider = selected.provider.clone();
                        config.ai.model = selected.name.clone();
                        app.provider = selected.provider.clone();
                        app.model = selected.name.clone();
                        *api_key = config.get_secure_api_key(&selected.provider).unwrap_or_default();
                        app.add_message("system", &format!("Switched provider and model to: {} ◇ {}", selected.provider.to_uppercase(), selected.label));
                        app.mode = AppMode::Normal;

                        // Track recent models (keep last 3)
                        let model_key = format!("{}:{}", selected.provider, selected.name);
                        app.recent_models.retain(|m| m != &model_key);
                        app.recent_models.insert(0, model_key);
                        app.recent_models.truncate(3);

                        if let Some(ref path) = app.config_path {
                            let _ = config.save_to_file(path);
                        }
                    }
                }
                _ => {}
            }
        }
        AppMode::SelectProviderDialog { selected_index } => {
            match key.code {
                KeyCode::Esc => {
                    // Return to model dialog
                    app.mode = AppMode::SelectModelDialog {
                        search_query: String::new(),
                        selected_index: 0,
                    };
                }
                KeyCode::Up => {
                    *selected_index = if *selected_index == 0 { 3 } else { *selected_index - 1 };
                }
                KeyCode::Down => {
                    *selected_index = (*selected_index + 1) % 4;
                }
                KeyCode::Enter => {
                    let providers = ["gemini", "anthropic", "openai", "openrouter"];
                    let selected_provider = providers[*selected_index % 4].to_string();
                    app.mode = AppMode::ApiKeyEntry {
                        provider: selected_provider,
                        key_input: String::new(),
                        cursor_position: 0,
                    };
                }
                _ => {}
            }
        }
        AppMode::ApiKeyEntry { provider, key_input, cursor_position } => {
            let provider_saved = provider.clone();
            let key_saved = key_input.clone();
            match key.code {
                KeyCode::Esc => {
                    app.mode = AppMode::SelectProviderDialog {
                        selected_index: 0,
                    };
                    return;
                }
                KeyCode::Enter => {
                    let key = key_saved.trim().to_string();
                    if !key.is_empty() {
                        match Config::save_secure_api_key(&provider_saved, &key) {
                            Ok(()) => {
                                app.refresh_configured_providers(config);
                                app.add_message("system", &format!("✓ API key saved for {}", provider_saved));
                                if config.ai.provider == provider_saved {
                                    *api_key = key;
                                }
                            }
                            Err(e) => {
                                app.add_message("system", &format!("Failed to save key: {}", e));
                            }
                        }
                        if let Some(ref path) = app.config_path {
                            let _ = config.save_to_file(path);
                        }
                    }
                    app.mode = AppMode::SelectProviderDialog {
                        selected_index: 0,
                    };
                    return;
                }
                KeyCode::Backspace => {
                    if *cursor_position > 0 {
                        let mut chars: Vec<char> = key_input.chars().collect();
                        chars.remove(*cursor_position - 1);
                        *key_input = chars.into_iter().collect();
                        *cursor_position = cursor_position.saturating_sub(1);
                    }
                }
                KeyCode::Delete => {
                    let mut chars: Vec<char> = key_input.chars().collect();
                    if *cursor_position < chars.len() {
                        chars.remove(*cursor_position);
                        *key_input = chars.into_iter().collect();
                    }
                }
                KeyCode::Left => {
                    *cursor_position = cursor_position.saturating_sub(1);
                }
                KeyCode::Right => {
                    let max = key_input.chars().count();
                    if *cursor_position < max {
                        *cursor_position += 1;
                    }
                }
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match c {
                            'u' | 'U' => {
                                key_input.clear();
                                *cursor_position = 0;
                            }
                            _ => {}
                        }
                    } else {
                        let mut chars: Vec<char> = key_input.chars().collect();
                        chars.insert(*cursor_position, c);
                        *key_input = chars.into_iter().collect();
                        *cursor_position += 1;
                    }
                }
                KeyCode::Home => {
                    *cursor_position = 0;
                }
                KeyCode::End => {
                    *cursor_position = key_input.chars().count();
                }
                _ => {}
            }
        }
        AppMode::CommandPalette { search_query, selected_index } => {
            match key.code {
                KeyCode::Esc => {
                    app.mode = AppMode::Normal;
                }
                KeyCode::Up => {
                    let cmds = CommandOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered: Vec<&CommandOption> = cmds.iter()
                        .filter(|c| c.name.to_lowercase().contains(&query) || c.description.to_lowercase().contains(&query))
                        .collect();
                    if !filtered.is_empty() {
                        *selected_index = if *selected_index == 0 { filtered.len() - 1 } else { *selected_index - 1 };
                    }
                }
                KeyCode::Down => {
                    let cmds = CommandOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered: Vec<&CommandOption> = cmds.iter()
                        .filter(|c| c.name.to_lowercase().contains(&query) || c.description.to_lowercase().contains(&query))
                        .collect();
                    if !filtered.is_empty() {
                        *selected_index = (*selected_index + 1) % filtered.len();
                    }
                }
                KeyCode::Backspace => {
                    search_query.pop();
                    *selected_index = 0;
                }
                KeyCode::Char(c) => {
                    search_query.push(c);
                    *selected_index = 0;
                }
                KeyCode::Enter => {
                    let cmds = CommandOption::get_all();
                    let query = search_query.to_lowercase();
                    let filtered: Vec<&CommandOption> = cmds.iter()
                        .filter(|c| c.name.to_lowercase().contains(&query) || c.description.to_lowercase().contains(&query))
                        .collect();
                    if !filtered.is_empty() {
                        let cmd_name = filtered[*selected_index % filtered.len()].name.clone();
                        app.mode = AppMode::Normal;
                        app.input = String::new();
                        app.cursor_position = 0;
                        // Execute the command directly
                        if cmd_name == "/models" {
                            open_model_dialog(app, config);
                        } else {
                            handle_command(&cmd_name, app, config, api_key, ai_tx).await;
                        }
                    }
                }
                _ => {}
            }
        }
        AppMode::Normal => {
            match key.code {
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match c {
                            'u' | 'U' => {
                                app.input.clear();
                                app.cursor_position = 0;
                            }
                            'd' | 'D' => {
                                if app.input.is_empty() {
                                    app.exit = true;
                                }
                            }
                            'p' | 'P' => {
                                app.mode = AppMode::CommandPalette {
                                    search_query: String::new(),
                                    selected_index: 0,
                                };
                            }
                            'l' | 'L' => {
                                app.messages.clear();
                                app.add_message("system", "Type /help for commands. Ask me anything about cybersecurity.");
                            }
                            'r' | 'R' => {
                                if let Some(prompt) = app.last_user_prompt.clone() {
                                    send_prompt(app, config, api_key, ai_tx, prompt);
                                } else {
                                    app.add_message("system", "No previous prompt to retry.");
                                }
                            }
                            's' | 'S' => {
                                export_conversation(app);
                            }
                            _ => {}
                        }
                    } else {
                        app.insert_char(c);
                        app.command_menu_index = 0;
                    }
                }
                KeyCode::Backspace => {
                    app.backspace_char();
                    app.command_menu_index = 0;
                }
                KeyCode::Delete => {
                    app.delete_char();
                    app.command_menu_index = 0;
                }
                KeyCode::Left => {
                    app.cursor_position = app.cursor_position.saturating_sub(1);
                }
                KeyCode::Right => {
                    let max_pos = app.input.chars().count();
                    if app.cursor_position < max_pos {
                        app.cursor_position += 1;
                    }
                }
                KeyCode::Home => {
                    app.cursor_position = 0;
                }
                KeyCode::End => {
                    app.cursor_position = app.input.chars().count();
                }
                KeyCode::Tab => {
                    let matches = app.get_autocomplete_commands();
                    if !matches.is_empty() {
                        app.command_menu_index = (app.command_menu_index + 1) % matches.len();
                    }
                }
                KeyCode::Up => {
                    let matches = app.get_autocomplete_commands();
                    if !matches.is_empty() {
                        // Navigate autocomplete dropdown menu with wrap
                        app.command_menu_index = if app.command_menu_index == 0 {
                            matches.len() - 1
                        } else {
                            app.command_menu_index - 1
                        };
                    } else if key.modifiers == KeyModifiers::CONTROL {
                        // Scroll chat viewport up
                        app.scroll_to_bottom = false;
                        app.scroll_offset = app.scroll_offset.saturating_sub(1);
                    } else {
                        // History navigation (Up)
                        if !app.history.is_empty() {
                            if app.history_index.is_none() {
                                app.temp_input = app.input.clone();
                                app.history_index = Some(app.history.len() - 1);
                            } else if let Some(idx) = app.history_index {
                                if idx > 0 {
                                    app.history_index = Some(idx - 1);
                                }
                            }
                            if let Some(idx) = app.history_index {
                                app.input = app.history[idx].clone();
                                app.cursor_position = app.input.chars().count();
                            }
                        }
                    }
                }
                KeyCode::Down => {
                    let matches = app.get_autocomplete_commands();
                    if !matches.is_empty() {
                        // Navigate autocomplete dropdown menu
                        app.command_menu_index = (app.command_menu_index + 1) % matches.len();
                    } else if key.modifiers == KeyModifiers::CONTROL {
                        // Scroll chat viewport down
                        app.scroll_offset = app.scroll_offset.saturating_add(1);
                    } else {
                        // History navigation (Down)
                        if let Some(idx) = app.history_index {
                            if idx + 1 < app.history.len() {
                                app.history_index = Some(idx + 1);
                                app.input = app.history[idx + 1].clone();
                                app.cursor_position = app.input.chars().count();
                            } else {
                                app.history_index = None;
                                app.input = app.temp_input.clone();
                                app.cursor_position = app.input.chars().count();
                            }
                        }
                    }
                }
                KeyCode::PageUp => {
                    app.scroll_to_bottom = false;
                    app.scroll_offset = app.scroll_offset.saturating_sub(5);
                }
                KeyCode::PageDown => {
                    app.scroll_offset = app.scroll_offset.saturating_add(5);
                }
                KeyCode::Enter => {
                    if key.modifiers.intersects(KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT) {
                        // Multi-line insertion
                        app.insert_char('\n');
                    } else {
                        let matches = app.get_autocomplete_commands();
                        if !matches.is_empty() && app.input.starts_with('/') {
                            // Autocomplete command selection on Enter
                            let selected = &matches[app.command_menu_index % matches.len()];
                            app.input = selected.name.clone();
                            app.cursor_position = app.input.chars().count();
                            app.command_menu_index = 0;

                            if app.input == "/models" {
                                app.input.clear();
                                app.cursor_position = 0;
                                open_model_dialog(app, config);
                                return;
                            }
                        }

                        let input = app.input.trim().to_string();
                        app.input.clear();
                        app.cursor_position = 0;
                        app.history_index = None;

                        if input.is_empty() {
                            return;
                        }

                        if app.history.last() != Some(&input) {
                            app.history.push(input.clone());
                        }

                        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") || input == "/exit" || input == "/quit" {
                            app.exit = true;
                            return;
                        }

                        if input.starts_with('/') {
                            handle_command(&input, app, config, api_key, ai_tx).await;
                            return;
                        }

                        send_prompt(app, config, api_key, ai_tx, input);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Open the model selection dialog, refreshing which providers currently have a usable key.
fn open_model_dialog(app: &mut App, config: &Config) {
    app.refresh_configured_providers(config);
    app.mode = AppMode::SelectModelDialog {
        search_query: String::new(),
        selected_index: 0,
    };
}

/// Send a prompt to the configured AI provider, streaming the response back through `ai_tx`.
/// Shared by the normal chat send path and `/retry`.
fn send_prompt(
    app: &mut App,
    config: &Config,
    api_key: &str,
    ai_tx: &mpsc::UnboundedSender<AiEvent>,
    prompt: String,
) {
    app.add_message("user", &prompt);
    app.loading = true;
    app.add_message("assistant", "");
    app.last_user_prompt = Some(prompt.clone());

    let tx = ai_tx.clone();
    let client = ai::build_client();
    let provider = config.ai.provider.clone();
    let model = config.ai.model.clone();
    let key = api_key.to_string();
    let handle = tokio::spawn(async move {
        let result = ai::stream_ai_response(
            &client, &provider, &model, &key, &prompt,
            &mut |chunk: &str| {
                let _ = tx.send(AiEvent::Chunk(chunk.to_string()));
            },
        ).await;
        match result {
            Ok(()) => { let _ = tx.send(AiEvent::Done); }
            Err(e) => { let _ = tx.send(AiEvent::Error(format!("{}", e))); }
        }
    });
    app.current_task = Some(handle);
}

/// Save the current conversation to a markdown file with a timestamp-based filename.
fn export_conversation(app: &mut App) {
    use chrono::Local;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("cypher_chat_{}.md", timestamp);
    let transcript = format_conversation(&app.messages);
    if transcript.is_empty() {
        app.add_message("system", "Nothing to export - conversation is empty.");
        return;
    }
    match std::fs::write(&filename, &transcript) {
        Ok(()) => app.add_message("system", &format!("✓ Conversation exported to {}", filename)),
        Err(e) => app.add_message("system", &format!("Export failed: {}", e)),
    }
}

async fn handle_command(
    input: &str,
    app: &mut App,
    config: &mut Config,
    _api_key: &mut String,
    ai_tx: &mpsc::UnboundedSender<AiEvent>,
) {
    let cmd = input.to_lowercase();
    if cmd == "/help" {
        app.add_message("system", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        app.add_message("system", "  Cypher CLI Commands");
        app.add_message("system", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        app.add_message("system", "  /models  - Switch AI provider and model");
        app.add_message("system", "  /scan    - Scan current directory for security issues");
        app.add_message("system", "  /copy    - Copy last response, code, or chat to clipboard");
        app.add_message("system", "  /clear   - Clear the conversation history");
        app.add_message("system", "  /retry   - Retry the last AI response");
        app.add_message("system", "  /export  - Export conversation to a markdown file");
        app.add_message("system", "  /upgrade - Upgrade Cypher CLI to the latest version");
        app.add_message("system", "  /help    - Display this help message");
        app.add_message("system", "  /exit    - Exit the session");
        app.add_message("system", "");
        app.add_message("system", "  Navigation:");
        app.add_message("system", "  Tab          - Cycle autocomplete suggestions");
        app.add_message("system", "  ↑/↓          - Navigate menus / input history");
        app.add_message("system", "  Ctrl+↑/↓     - Scroll chat viewport");
        app.add_message("system", "  Ctrl+L       - Clear conversation");
        app.add_message("system", "  Ctrl+R       - Retry last AI response");
        app.add_message("system", "  Ctrl+S       - Export conversation to file");
        app.add_message("system", "  Ctrl+P       - Open command palette");
        app.add_message("system", "  Ctrl+U       - Clear input line");
        app.add_message("system", "  Shift+Enter  - New line (multi-line input)");
        app.add_message("system", "  Esc          - Cancel AI response");
        app.add_message("system", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        app.add_message("system", "  Just type any security question to get started.");
    } else if cmd == "/clear" {
        app.messages.clear();
        app.add_message("system", "Type /help for commands. Ask me anything about cybersecurity.");
    } else if cmd == "/retry" {
        if let Some(prompt) = app.last_user_prompt.clone() {
            send_prompt(app, config, _api_key, ai_tx, prompt);
        } else {
            app.add_message("system", "No previous prompt to retry.");
        }
    } else if cmd == "/copy" || cmd.starts_with("/copy ") {
        let arg = cmd.strip_prefix("/copy ").unwrap_or("").trim();
        
        let last_assistant_msg = app.messages.iter()
            .filter(|m| m.role == "assistant")
            .last();

        match arg {
            "code" => {
                if let Some(msg) = last_assistant_msg {
                    if let Some(code) = extract_last_code_block(&msg.content) {
                        match copy_text_to_clipboard(&code) {
                            Ok(()) => app.add_message("system", "✓ Last code block copied to clipboard."),
                            Err(e) => app.add_message("system", &format!("Clipboard error: {}", e)),
                        }
                    } else {
                        app.add_message("system", "No code block found in the last AI response.");
                    }
                } else {
                    app.add_message("system", "No AI response to copy code from.");
                }
            }
            "chat" | "conversation" => {
                let transcript = format_conversation(&app.messages);
                if !transcript.is_empty() {
                    match copy_text_to_clipboard(&transcript) {
                        Ok(()) => app.add_message("system", "✓ Entire chat transcript copied to clipboard."),
                        Err(e) => app.add_message("system", &format!("Clipboard error: {}", e)),
                    }
                } else {
                    app.add_message("system", "No conversation history to copy.");
                }
            }
            "last" | "" => {
                if let Some(msg) = last_assistant_msg {
                    match copy_text_to_clipboard(&msg.content) {
                        Ok(()) => app.add_message("system", "✓ Last AI response copied to clipboard."),
                        Err(e) => app.add_message("system", &format!("Clipboard error: {}", e)),
                    }
                } else {
                    app.add_message("system", "No AI response to copy.");
                }
            }
            other => {
                app.add_message("system", &format!("Invalid copy argument: '{}'. Use: /copy [last|code|chat]", other));
            }
        }
    } else if cmd == "/scan" {
        app.loading = true;
        app.add_message("assistant", "Running real-time security scan...\n");
        let tx = ai_tx.clone();
        let path = std::env::current_dir().unwrap();
        let config_clone = config.clone();
        tokio::spawn(async move {
            let _ = tx.send(AiEvent::Chunk("Initializing scan engine...\n".to_string()));
            
            let mut rule_engine = crate::rules::RuleEngine::with_threshold(crate::rules::Severity::Low);
            if let Err(e) = rule_engine.register_rules(crate::rules::RuleLibrary::get_all_rules()) {
                let _ = tx.send(AiEvent::Error(format!("Failed to register rules: {}", e)));
                return;
            }

            let scanner = crate::scanner::Scanner::new(config_clone, rule_engine);
            match scanner.scan(&path).await {
                Ok(results) => {
                    let total_findings: usize = results.iter().map(|r| r.finding_count()).sum();
                    let mut summary = format!("Scan complete. Found {} security issues.\n\n", total_findings);
                    if total_findings > 0 {
                        summary.push_str("Summary of findings:\n");
                        for r in &results {
                            for m in &r.matches {
                                summary.push_str(&format!(
                                    "  - {} in {}:{} ({} severity)\n",
                                    r.rule.name,
                                    m.file.file_name().unwrap_or_default().to_string_lossy(),
                                    m.line,
                                    r.rule.severity.to_string().to_uppercase()
                                ));
                            }
                        }
                    } else {
                        summary.push_str("No security issues found. Great job!\n");
                    }
                    let _ = tx.send(AiEvent::Chunk(summary));
                }
                Err(e) => {
                    let _ = tx.send(AiEvent::Error(format!("Scan failed: {}", e)));
                }
            }
            let _ = tx.send(AiEvent::Done);
        });
    } else if cmd == "/models" {
        open_model_dialog(app, config);
    } else if cmd == "/export" || cmd.starts_with("/export ") {
        export_conversation(app);
    } else if cmd == "/upgrade" {
        app.add_message("system", "To upgrade Cypher CLI, run this command in your terminal:\n  cypher upgrade\n\nThe TUI will show progress bars and version info during the upgrade. Restart the TUI after upgrading.");
    } else {
        app.add_message("system", &format!("Unknown command: {}\nType /help for available commands.", cmd));
    }
}

/// Helper to extract code blocks from a markdown text.
/// If multiple code blocks exist, it returns the last one.
/// If no code blocks exist, it returns None.
fn extract_last_code_block(markdown: &str) -> Option<String> {
    let mut code_blocks = Vec::new();
    let mut current_block = String::new();
    let mut inside_block = false;

    for line in markdown.lines() {
        if line.trim().starts_with("```") {
            if inside_block {
                // End of a code block
                code_blocks.push(current_block.trim_end().to_string());
                current_block.clear();
                inside_block = false;
            } else {
                // Start of a code block
                inside_block = true;
            }
        } else if inside_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    code_blocks.pop()
}

fn format_conversation(messages: &[ChatMessage]) -> String {
    let mut transcript = String::new();
    for msg in messages {
        let role_label = match msg.role.as_str() {
            "user" => "User",
            "assistant" => "Cypher AI",
            "system" => "System",
            other => other,
        };
        transcript.push_str(&format!("[{}]\n{}\n\n", role_label, msg.content));
    }
    transcript.trim_end().to_string()
}

fn copy_text_to_clipboard(text: &str) -> std::result::Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| format!("{}", e))?;
    cb.set_text(text.to_string()).map_err(|e| format!("{}", e))?;
    Ok(())
}

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    draw_header(frame, chunks[0], app);
    draw_messages(frame, chunks[1], app);
    draw_input(frame, chunks[2], app);

    // Draw Drop-up Autocomplete Command Menu if typing /
    if app.mode == AppMode::Normal && app.input.starts_with('/') {
        let matches = app.get_autocomplete_commands();
        if !matches.is_empty() {
            let input_area = chunks[2];
            let max_height = input_area.y as usize;
            let menu_height = (matches.len() + 2).min(10).min(max_height) as u16;
            if menu_height > 2 {
                let menu_area = Rect {
                    x: input_area.x.saturating_add(2),
                    y: input_area.y.saturating_sub(menu_height),
                    width: 60.min(input_area.width.saturating_sub(4)),
                    height: menu_height,
                };

            frame.render_widget(Clear, menu_area);

            let mut list_lines = Vec::new();
            for (idx, cmd) in matches.iter().enumerate() {
                let is_selected = idx == app.command_menu_index % matches.len();
                let line_str = format!("  {:<12} {}", cmd.name, cmd.description);
                // Highlight color matching orange/brown background
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Rgb(240, 140, 60)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200))
                };
                list_lines.push(Line::from(vec![Span::styled(line_str, style)]));
            }

            let dropdown_widget = Paragraph::new(Text::from(list_lines))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(80, 80, 90)))
                    .bg(Color::Rgb(20, 20, 25)));

            frame.render_widget(dropdown_widget, menu_area);
            }
        }
    }

    // Draw Centered Model Selection Modal Dialog
    if let AppMode::SelectModelDialog { search_query, selected_index } = &app.mode {
        let size = frame.size();
        let dialog_width = 60.min(size.width);
        let dialog_height = (size.height / 2).clamp(14, 26).min(size.height);
        let dialog_area = Rect {
            x: size.x + (size.width.saturating_sub(dialog_width)) / 2,
            y: size.y + (size.height.saturating_sub(dialog_height)) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 90)))
            .bg(Color::Rgb(20, 20, 25));
        frame.render_widget(block, dialog_area);

        let inner_width = dialog_width.saturating_sub(2) as usize;
        let mut content = Vec::new();

        let configured_count = app.configured_providers.len();
        let header_title = format!(" Select model  ({}/4 configured)", configured_count);
        let header_esc = "esc ";
        let pad_len = inner_width.saturating_sub(header_title.len() + header_esc.len());
        content.push(Line::from(vec![
            Span::styled(&header_title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(pad_len)),
            Span::styled(header_esc, Style::default().fg(Color::DarkGray)),
        ]));
        content.push(Line::from(""));

        // Search bar
        let search_style = if search_query.is_empty() {
            Style::default().fg(Color::DarkGray).italic()
        } else {
            Style::default().fg(Color::White)
        };
        let placeholder = if search_query.is_empty() { "Search" } else { search_query };
        content.push(Line::from(vec![
            Span::styled("  Search  ", Style::default().fg(Color::DarkGray)),
            Span::styled(placeholder, search_style),
            Span::styled("█", Style::default().fg(Color::Rgb(240, 140, 60))),
        ]));
        content.push(Line::from(""));

        let query = search_query.to_lowercase();
        let all_models = ModelOption::get_all();
        let has_search = !query.is_empty();

        // Collect provider groups with their models
        struct ProviderGroup<'a> {
            provider: &'a str,
            display_name: &'a str,
            models: Vec<&'a ModelOption>,
        }

        let provider_groups_data = [
            ("anthropic", "Anthropic"),
            ("openai", "OpenAI"),
            ("gemini", "Gemini"),
            ("openrouter", "OpenRouter"),
        ];

        let mut groups: Vec<ProviderGroup> = Vec::new();
        for (provider_id, display_name) in &provider_groups_data {
            let models: Vec<&ModelOption> = all_models.iter()
                .filter(|m| {
                    m.provider == *provider_id &&
                    (has_search && !has_search || m.label.to_lowercase().contains(&query) || m.provider.to_lowercase().contains(&query))
                })
                .collect();
            if !models.is_empty() || has_search {
                // In search mode, include groups even if empty (they won't show items)
                if !models.is_empty() {
                    groups.push(ProviderGroup { provider: provider_id, display_name, models });
                }
            } else {
                groups.push(ProviderGroup { provider: provider_id, display_name, models });
            }
        }

        // Build flat list for selection indexing
        struct FlatEntry {
            is_header: bool,
            model_idx: usize, // index into the model within its group
            group_idx: usize,
        }

        let mut flat_list: Vec<FlatEntry> = Vec::new();
        let mut category_count = 0;

        // Recent section
        let mut recent_entries: Vec<(&ModelOption, usize)> = Vec::new();
        if !has_search && !app.recent_models.is_empty() {
            for recent_key in &app.recent_models {
                if let Some((provider, name)) = recent_key.split_once(':') {
                    if let Some(m) = all_models.iter().find(|m| m.provider == provider && m.name == name) {
                        recent_entries.push((m, 0));
                    }
                }
            }
        }

        // Build flat list: recent items, then headers + items per group
        let recent_count = recent_entries.len();
        for i in 0..recent_count {
            flat_list.push(FlatEntry { is_header: false, model_idx: i, group_idx: 0 });
            category_count += 1;
        }

        for (gi, group) in groups.iter().enumerate() {
            if group.models.is_empty() {
                continue;
            }
            flat_list.push(FlatEntry { is_header: true, model_idx: 0, group_idx: gi });
            for mi in 0..group.models.len() {
                flat_list.push(FlatEntry { is_header: false, model_idx: mi, group_idx: gi });
                category_count += 1;
            }
        }

        let selection = *selected_index % flat_list.len().max(1);

        // Render recent section
        if !recent_entries.is_empty() {
            content.push(Line::from(vec![
                Span::styled("  Recent", Style::default().fg(Color::Rgb(100, 100, 200)).add_modifier(Modifier::BOLD)),
            ]));
            for (model, _) in &recent_entries {
                let _global_idx = flat_list.iter().position(|f| !f.is_header && f.model_idx == 0).unwrap_or(0);
                let is_active = app.provider == model.provider && app.model == model.name;
                let is_sel = flat_list.iter().position(|f| !f.is_header).unwrap_or(0) == selection;
                let connected = app.configured_providers.contains(&model.provider.to_string());
                let badge = if connected { "✓".to_string() } else { " ".to_string() };
                let active_dot = if is_active { "●".cyan().to_string() } else { " ".to_string() };
                let prefix = if is_sel { " • " } else { "   " };
                let left_str = format!("{}{}{} {}", prefix, active_dot, badge, model.label);
                let right_str = format!("{} ", model.tag);
                let text_pad = inner_width.saturating_sub(left_str.chars().count() + right_str.chars().count() + 1);
                let line_str = format!("{}{}{}", left_str, " ".repeat(text_pad), right_str);
                let style = if is_sel {
                    Style::default().fg(Color::Black).bg(Color::Rgb(240, 140, 60)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200))
                };
                content.push(Line::from(vec![Span::styled(line_str, style)]));
            }
            content.push(Line::from(""));
        }

        // Render groups
        if flat_list.is_empty() {
            content.push(Line::from(vec![
                Span::styled("    No models match your search.", Style::default().fg(Color::DarkGray).italic()),
            ]));
        } else {
            for (gi, group) in groups.iter().enumerate() {
                if group.models.is_empty() {
                    continue;
                }
                let connected = app.configured_providers.contains(group.provider);
                let status_icon = if connected { "✓" } else { "✗" };
                let section_line = format!("── {} {} ──", group.display_name, status_icon);
                let section_style = if connected {
                    Style::default().fg(Color::Rgb(120, 200, 120))
                } else {
                    Style::default().fg(Color::Rgb(150, 150, 150))
                };
                content.push(Line::from(vec![Span::styled(section_line, section_style)]));

                for model in &group.models {
                    let is_active = app.provider == model.provider && app.model == model.name;
                    // Find this model's position in flat_list
                    let mut global_idx = 0;
                    let mut found = false;
                    for (fi, entry) in flat_list.iter().enumerate() {
                        if !entry.is_header && entry.group_idx == gi {
                            if global_idx == category_count {
                                break;
                            }
                            if groups[gi].models[entry.model_idx].name == model.name {
                                global_idx = fi;
                                found = true;
                                break;
                            }
                            global_idx += 1;
                        }
                    }
                    let is_sel = found && global_idx == selection;
                    let connected = app.configured_providers.contains(&model.provider.to_string());
                    let badge = if connected { "✓".to_string() } else { " ".to_string() };
                    let active_dot = if is_active { "●".cyan().to_string() } else { " ".to_string() };
                    let prefix = if is_sel { " • " } else { "   " };
                    let left_str = format!("{}{}{} {}", prefix, active_dot, badge, model.label);
                    let right_str = format!("{} ", model.tag);
                    let text_pad = inner_width.saturating_sub(left_str.chars().count() + right_str.chars().count() + 1);
                    let line_str = format!("{}{}{}", left_str, " ".repeat(text_pad), right_str);
                    let style = if is_sel {
                        Style::default().fg(Color::Black).bg(Color::Rgb(240, 140, 60)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Rgb(200, 200, 200))
                    };
                    content.push(Line::from(vec![Span::styled(line_str, style)]));
                }
                content.push(Line::from(""));
            }
        }

        // Fill remaining height with empty space before footer
        let current_len = content.len() + 2;
        let pad_lines = (dialog_height as usize).saturating_sub(current_len + 1);
        for _ in 0..pad_lines {
            content.push(Line::from(""));
        }

        // Footnote hint
        content.push(Line::from(vec![
            Span::styled("  ✓ configured  ↑↓ navigate  ↵ select  ctrl+a add provider  esc", Style::default().fg(Color::DarkGray)),
        ]));

        let dialog_widget = Paragraph::new(Text::from(content))
            .bg(Color::Rgb(20, 20, 25));

        let inner_dialog_area = Rect {
            x: dialog_area.x + 1,
            y: dialog_area.y + 1,
            width: dialog_area.width.saturating_sub(2),
            height: dialog_area.height.saturating_sub(2),
        };
        frame.render_widget(dialog_widget, inner_dialog_area);
    }

    // Draw Provider Selection Dialog
    if let AppMode::SelectProviderDialog { selected_index } = &app.mode {
        let size = frame.size();
        let dialog_w = 50.min(size.width);
        let dialog_h = 14.min(size.height);
        let dialog_area = Rect {
            x: size.x + (size.width.saturating_sub(dialog_w)) / 2,
            y: size.y + (size.height.saturating_sub(dialog_h)) / 2,
            width: dialog_w,
            height: dialog_h,
        };

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 90)))
            .bg(Color::Rgb(20, 20, 25));
        frame.render_widget(block, dialog_area);

        let inner_w = dialog_w.saturating_sub(2) as usize;
        let mut p_content = Vec::new();

        let provider_list = [
            ("gemini", "Gemini", "GEMINI_API_KEY"),
            ("anthropic", "Anthropic", "ANTHROPIC_API_KEY"),
            ("openai", "OpenAI", "OPENAI_API_KEY"),
            ("openrouter", "OpenRouter", "OPENROUTER_API_KEY"),
        ];

        let header_line = Line::from(vec![
            Span::styled(" Configure Provider ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(inner_w.saturating_sub(19 + 4))),
            Span::styled("esc ", Style::default().fg(Color::DarkGray)),
        ]);
        p_content.push(header_line);
        p_content.push(Line::from(""));

        for (i, (id, name, env_var)) in provider_list.iter().enumerate() {
            let is_sel = i == *selected_index % provider_list.len();
            let configured = app.configured_providers.contains(*id);
            let badge = if configured { "✓".green().to_string() } else { " ".to_string() };
            let prefix = if is_sel { " • " } else { "   " };
            let status = if configured { "configured" } else { "not set" };
            let left_str = format!("{}{} {}", prefix, badge, name);
            let right_str = format!("{} ({})", env_var, status);
            let text_pad = inner_w.saturating_sub(left_str.chars().count() + right_str.chars().count() + 1);
            let line_str = format!("{}{}{}", left_str, " ".repeat(text_pad), right_str);
            let style = if is_sel {
                Style::default().fg(Color::Black).bg(Color::Rgb(240, 140, 60)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(200, 200, 200))
            };
            p_content.push(Line::from(vec![Span::styled(line_str, style)]));
        }

        let current = p_content.len() + 2;
        let pad = (dialog_h as usize).saturating_sub(current + 1);
        for _ in 0..pad {
            p_content.push(Line::from(""));
        }

        p_content.push(Line::from(vec![
            Span::styled("  ↑↓ navigate  ↵ configure  esc back", Style::default().fg(Color::DarkGray)),
        ]));

        let widget = Paragraph::new(Text::from(p_content))
            .bg(Color::Rgb(20, 20, 25));
        let inner = Rect {
            x: dialog_area.x + 1,
            y: dialog_area.y + 1,
            width: dialog_area.width.saturating_sub(2),
            height: dialog_area.height.saturating_sub(2),
        };
        frame.render_widget(widget, inner);
    }

    // Draw API Key Entry Dialog
    if let AppMode::ApiKeyEntry { provider, key_input, cursor_position } = &app.mode {
        let size = frame.size();
        let dialog_w = 55.min(size.width);
        let dialog_h = 8.min(size.height);
        let dialog_area = Rect {
            x: size.x + (size.width.saturating_sub(dialog_w)) / 2,
            y: size.y + (size.height.saturating_sub(dialog_h)) / 2,
            width: dialog_w,
            height: dialog_h,
        };

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 90)))
            .bg(Color::Rgb(20, 20, 25));
        frame.render_widget(block, dialog_area);

        let inner_w = dialog_w.saturating_sub(2) as usize;
        let mut content = Vec::new();

        let env_var = match provider.as_str() {
            "gemini" => "GEMINI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            _ => "API_KEY",
        };

        content.push(Line::from(vec![
            Span::styled(" API Key: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(provider, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        content.push(Line::from(vec![
            Span::styled(format!(" Set {} env var or enter key below:", env_var), Style::default().fg(Color::DarkGray)),
        ]));
        content.push(Line::from(""));

        let input_style = if key_input.is_empty() {
            Style::default().fg(Color::DarkGray).italic()
        } else {
            Style::default().fg(Color::Rgb(200, 200, 200))
        };
        let display_text = if key_input.is_empty() {
            "Paste API key here...".to_string()
        } else {
            let len = key_input.chars().count();
            if len > 4 {
                format!("{}...{}", "█".repeat(20.min(len - 4)), &key_input[len-4..])
            } else {
                "█".repeat(len)
            }
        };

        content.push(Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Green)),
            Span::styled(display_text, input_style),
        ]));
        content.push(Line::from(""));

        content.push(Line::from(vec![
            Span::styled("  ↵ save  esc cancel", Style::default().fg(Color::DarkGray)),
        ]));

        let widget = Paragraph::new(Text::from(content))
            .bg(Color::Rgb(20, 20, 25));
        let inner = Rect {
            x: dialog_area.x + 1,
            y: dialog_area.y + 1,
            width: dialog_area.width.saturating_sub(2),
            height: dialog_area.height.saturating_sub(2),
        };
        frame.render_widget(widget, inner);

        // Set cursor position for key input
        let prompt_offset = "  > ".chars().count();
        let cursor_col = (*cursor_position).min(inner_w.saturating_sub(prompt_offset + 1));
        frame.set_cursor(inner.x + (prompt_offset + cursor_col) as u16, inner.y + 3);
    }

    // Draw Command Palette Dialog (Ctrl+P)
    if let AppMode::CommandPalette { search_query, selected_index } = &app.mode {
        let size = frame.size();
        let palette_width = 50.min(size.width);
        let palette_height = 14.min(size.height);
        let palette_area = Rect {
            x: size.x + (size.width.saturating_sub(palette_width)) / 2,
            y: size.y + (size.height.saturating_sub(palette_height)) / 2,
            width: palette_width,
            height: palette_height,
        };

        frame.render_widget(Clear, palette_area);

        let block = Block::default()
            .title(" Command Palette ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 90)))
            .bg(Color::Rgb(20, 20, 25));
        frame.render_widget(block, palette_area);

        let _inner_w = palette_width.saturating_sub(2) as usize;
        let mut p_content = Vec::new();

        // Search bar
        let p_placeholder = if search_query.is_empty() { "Search commands..." } else { search_query };
        let p_style = if search_query.is_empty() {
            Style::default().fg(Color::DarkGray).italic()
        } else {
            Style::default().fg(Color::White)
        };
        p_content.push(Line::from(vec![
            Span::styled(" ▸ ", Style::default().fg(Color::Green)),
            Span::styled(p_placeholder, p_style),
        ]));
        p_content.push(Line::from(""));

        // Filtered commands
        let p_query = search_query.to_lowercase();
        let p_cmds = CommandOption::get_all();
        let p_filtered: Vec<&CommandOption> = p_cmds.iter()
            .filter(|c| c.name.to_lowercase().contains(&p_query) || c.description.to_lowercase().contains(&p_query))
            .collect();

        if p_filtered.is_empty() {
            p_content.push(Line::from(vec![
                Span::styled("  No commands match.", Style::default().fg(Color::DarkGray).italic()),
            ]));
        } else {
            for (idx, cmd) in p_filtered.iter().enumerate() {
                let is_sel = idx == *selected_index % p_filtered.len();
                let prefix = if is_sel { " • " } else { "   " };
                let line_str = format!("{}{:<12} {}", prefix, cmd.name, cmd.description);
                let p_style = if is_sel {
                    Style::default().fg(Color::Black).bg(Color::Rgb(240, 140, 60)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200))
                };
                p_content.push(Line::from(vec![Span::styled(line_str, p_style)]));
            }
        }

        let p_current = p_content.len() + 2;
        let p_pad = (palette_height as usize).saturating_sub(p_current + 1);
        for _ in 0..p_pad {
            p_content.push(Line::from(""));
        }

        p_content.push(Line::from(vec![
            Span::styled("  ↑↓ navigate  ↵ execute  esc close", Style::default().fg(Color::DarkGray)),
        ]));

        let palette_widget = Paragraph::new(Text::from(p_content))
            .bg(Color::Rgb(20, 20, 25));
        let inner_palette = Rect {
            x: palette_area.x + 1,
            y: palette_area.y + 1,
            width: palette_area.width.saturating_sub(2),
            height: palette_area.height.saturating_sub(2),
        };
        frame.render_widget(palette_widget, inner_palette);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let status = if app.loading {
        format!("● {} ◇ {} [thinking...]", app.provider, app.model)
    } else {
        format!("● {} ◇ {}", app.provider, app.model)
    };
    let mode_str = match app.mode {
        AppMode::Normal => "",
        AppMode::CommandPalette { .. } => " [COMMANDS]",
        AppMode::SelectModelDialog { .. } => " [SELECT MODEL]",
        AppMode::SelectProviderDialog { .. } => " [PROVIDERS]",
        AppMode::ApiKeyEntry { .. } => " [API KEY]",
    };
    let header = Line::from(vec![
        Span::styled(" Cypher ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("v0.1.1", Style::default().fg(Color::DarkGray)),
        Span::raw(" │ "),
        Span::styled(status, Style::default().fg(Color::Cyan)),
        Span::styled(mode_str, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(header).style(Style::default().bg(Color::Reset)), area);
}

fn draw_messages(frame: &mut Frame, area: Rect, app: &App) {
    if app.messages.is_empty() {
        let welcome = Text::from(vec![
            Line::from(vec![
                Span::styled("\n  ⚡ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("Cypher Security AI", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Ask me anything about cybersecurity,", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("  or type /help for available commands.", Style::default().fg(Color::DarkGray))),
            Line::from(""),
        ]);
        frame.render_widget(
            Paragraph::new(welcome).style(Style::default().bg(Color::Reset)),
            area,
        );
        return;
    }

    let mut text = Text::default();
    for msg in &app.messages {
        let (label, label_style, style) = match msg.role.as_str() {
            "user" => (
                "  YOU ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Rgb(150, 200, 255)).bg(Color::Reset),
            ),
            "assistant" => (
                "  CYPHER ",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Rgb(200, 200, 220)).bg(Color::Reset),
            ),
            _ => (
                "  ● ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Yellow).bg(Color::Reset),
            ),
        };
        text.extend(Text::from(Line::from(vec![
            Span::styled(label, label_style),
        ])));
        for line in msg.content.lines() {
            text.extend(Text::from(Line::from(vec![
                Span::styled(format!("  {}", line), style),
            ])));
        }
        text.extend(Text::from(Line::from("")));
    }

    if app.loading {
        text.extend(Text::from(Line::from(vec![
            Span::styled("  ⏳ thinking...", Style::default().fg(Color::Cyan).italic()),
        ])));
    }

    let msg_area = Paragraph::new(text)
        .scroll((app.scroll_offset, 0))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Reset));
    frame.render_widget(msg_area, area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let input_style = if app.loading {
        Style::default().fg(Color::DarkGray).bg(Color::Reset)
    } else {
        Style::default().fg(Color::White).bg(Color::Reset)
    };

    let prompt_char = match app.mode {
        AppMode::Normal => " ▸ ",
        AppMode::SelectModelDialog { .. } => " Search model ▸ ",
        AppMode::SelectProviderDialog { .. } => " Select provider ▸ ",
        AppMode::ApiKeyEntry { .. } => " Enter API key ▸ ",
        AppMode::CommandPalette { .. } => " Search cmd ▸ ",
    };

    let prompt = Line::from(vec![
        Span::styled(prompt_char, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(&app.input, input_style),
    ]);

    let input_widget = Paragraph::new(prompt)
        .style(Style::default().bg(Color::Reset))
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 50))));
    frame.render_widget(input_widget, area);

    // Only set cursor position in Normal Mode (Model selector uses internal cursor box)
    if let AppMode::Normal = app.mode {
        let prompt_len = prompt_char.chars().count();
        let width = area.width.saturating_sub(prompt_len as u16 + 1) as usize;
        let cursor_col = app.cursor_position.min(width);
        frame.set_cursor(area.x + (prompt_len + cursor_col) as u16, area.y + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers, KeyCode, KeyEventState};

    fn make_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn make_char_event(c: char) -> KeyEvent {
        make_key_event(KeyCode::Char(c))
    }

    #[tokio::test]
    async fn test_tui_input_editor_and_cursor() {
        let mut app = App::new("gemini".to_string(), "gemini-2.0-flash".to_string());
        let mut config = Config::default();
        let mut api_key = "test_key".to_string();
        let (ai_tx, _ai_rx) = mpsc::unbounded_channel();

        // Type a word
        handle_key(make_char_event('h'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('e'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('l'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('l'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('o'), &mut app, &mut config, &mut api_key, &ai_tx).await;

        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 5);

        // Move cursor left twice
        handle_key(make_key_event(KeyCode::Left), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_key_event(KeyCode::Left), &mut app, &mut config, &mut api_key, &ai_tx).await;
        assert_eq!(app.cursor_position, 3);

        // Type a slash (single slash!)
        handle_key(make_char_event('/'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        assert_eq!(app.input, "hel/lo");
        assert_eq!(app.cursor_position, 4);

        // Backspace the slash
        handle_key(make_key_event(KeyCode::Backspace), &mut app, &mut config, &mut api_key, &ai_tx).await;
        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 3);

        // Move cursor to start using Home
        handle_key(make_key_event(KeyCode::Home), &mut app, &mut config, &mut api_key, &ai_tx).await;
        assert_eq!(app.cursor_position, 0);

        // Press delete (removes 'h')
        handle_key(make_key_event(KeyCode::Delete), &mut app, &mut config, &mut api_key, &ai_tx).await;
        assert_eq!(app.input, "ello");
        assert_eq!(app.cursor_position, 0);
    }

    #[tokio::test]
    async fn test_stateful_model_switching() {
        let mut app = App::new("gemini".to_string(), "gemini-2.0-flash".to_string());
        let mut config = Config::default();
        let mut api_key = "test_key".to_string();
        let (ai_tx, _ai_rx) = mpsc::unbounded_channel();

        // 1. Initially mode is Normal
        assert_eq!(app.mode, AppMode::Normal);

        // 2. Type '/models' and hit Enter
        for c in "/models".chars() {
            handle_key(make_char_event(c), &mut app, &mut config, &mut api_key, &ai_tx).await;
        }
        handle_key(make_key_event(KeyCode::Enter), &mut app, &mut config, &mut api_key, &ai_tx).await;

        // Mode should transition to SelectModelDialog
        assert!(matches!(app.mode, AppMode::SelectModelDialog { .. }));

        // 3. Type 'Deep' in search query and hit Enter
        handle_key(make_char_event('D'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('e'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('e'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_char_event('p'), &mut app, &mut config, &mut api_key, &ai_tx).await;
        handle_key(make_key_event(KeyCode::Enter), &mut app, &mut config, &mut api_key, &ai_tx).await;

        // Mode should revert to Normal, and model updated to DeepSeek Chat (via OpenRouter)
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.provider, "openrouter");
        assert_eq!(app.model, "deepseek/deepseek-chat");
    }

    #[test]
    fn test_extract_last_code_block() {
        let markdown = "Hello, here is some code:\n```rust\nfn main() {}\n```\nAnd another one:\n```python\nprint('hello')\n```\nDone.";
        let code = extract_last_code_block(markdown);
        assert_eq!(code, Some("print('hello')".to_string()));

        let markdown_no_code = "Just simple text without code blocks.";
        let code_none = extract_last_code_block(markdown_no_code);
        assert_eq!(code_none, None);
    }

    #[test]
    fn test_format_conversation() {
        let messages = vec![
            ChatMessage { role: "user".to_string(), content: "hello".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "hi there".to_string() },
        ];
        let transcript = format_conversation(&messages);
        assert_eq!(transcript, "[User]\nhello\n\n[Cypher AI]\nhi there");
    }
}
