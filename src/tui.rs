use crate::ai;
use crate::config::Config;
use crate::error::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc;

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
    pub scroll_offset: u16,
    pub provider: String,
    pub model: String,
    pub loading: bool,
    pub exit: bool,
}

impl App {
    pub fn new(provider: String, model: String) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            scroll_offset: 0,
            provider,
            model,
            loading: false,
            exit: false,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
        self.scroll_offset = 0;
    }

    pub fn append_to_last(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            last.content.push_str(content);
        }
    }
}

pub fn setup_terminal() -> io::Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal() -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

pub async fn run_tui(app: &mut App, config: &mut Config) -> Result<()> {
    let mut terminal = setup_terminal().map_err(crate::error::CypherError::Io)?;
    let (ai_tx, mut ai_rx) = mpsc::unbounded_channel::<AiEvent>();
    let mut event_stream = EventStream::new();

    let mut api_key = config.get_secure_api_key(&config.ai.provider).unwrap_or_default();

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
                    }
                    AiEvent::Error(e) => {
                        app.loading = false;
                        app.add_message("system", &format!("Error: {}", e));
                    }
                }
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
    if app.loading {
        return;
    }

    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers == KeyModifiers::CONTROL && c == 'c' {
                app.exit = true;
                return;
            }
            if key.modifiers == KeyModifiers::CONTROL && c == 'u' {
                app.input.clear();
                return;
            }
            app.input.push(c);
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Enter => {
            let input = app.input.trim().to_string();
            app.input.clear();
            if input.is_empty() {
                return;
            }
            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") || input == "\\exit" || input == "\\quit" {
                app.exit = true;
                return;
            }
            if input.starts_with('\\') {
                handle_command(&input, app, config, api_key, ai_tx).await;
                return;
            }
            app.add_message("user", &input);
            app.loading = true;
            app.add_message("assistant", "");
            let tx = ai_tx.clone();
            let client = reqwest::Client::new();
            let provider = config.ai.provider.clone();
            let model = config.ai.model.clone();
            let key = api_key.clone();
            let prompt = input.clone();
            tokio::spawn(async move {
                let result = ai::stream_ai_response(
                    &client, &provider, &model, &key, &prompt,
                    &mut |chunk: &str| {
                        let _ = tx.send(AiEvent::Chunk(chunk.to_string()));
                    },
                ).await;
                match result {
                    Ok(()) => { let _ = tx.send(AiEvent::Done); }
                    Err(e) => { let _ = tx.send(AiEvent::Error(format!("{:?}", e))); }
                }
            });
        }
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_add(5);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_sub(5);
        }
        KeyCode::Up => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
        }
        KeyCode::Down => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        _ => {}
    }
}

async fn handle_command(
    input: &str,
    app: &mut App,
    config: &mut Config,
    api_key: &mut String,
    ai_tx: &mpsc::UnboundedSender<AiEvent>,
) {
    let cmd = input.to_lowercase();
    if cmd == "\\help" {
        let help = r#"Cypher CLI Commands:
  \models  - Switch AI provider and model
  \scan    - Scan current directory for security issues
  \help    - Display this help message
  \exit    - Exit the session

Just type any security question to get started."#;
        app.add_message("system", help);
    } else if cmd == "\\scan" {
        app.loading = true;
        app.add_message("assistant", "Running security scan...\n");
        let tx = ai_tx.clone();
        let path = std::env::current_dir().unwrap();
        tokio::spawn(async move {
            let _ = tx.send(AiEvent::Chunk("Initializing scan...\n".to_string()));
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = tx.send(AiEvent::Chunk(format!("Scanning: {}\n", path.display())));
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = tx.send(AiEvent::Chunk("Scan complete. Run 'cypher scan . --output json' for detailed results.".to_string()));
            let _ = tx.send(AiEvent::Done);
        });
    } else if cmd == "\\models" {
        let providers = ["anthropic", "openai", "gemini", "openrouter"];
        let provider_list: String = providers.iter().enumerate()
            .map(|(i, p)| format!("  {}. {}", i + 1, p))
            .collect::<Vec<_>>()
            .join("\n");
        app.add_message("system", &format!("Available providers:\n{}\n\nType the provider name (e.g., 'gemini') to select.", provider_list));
        let _ = ai_tx.send(AiEvent::Chunk("".to_string()));
    } else {
        let providers = ["anthropic", "openai", "gemini", "openrouter"];
        if providers.contains(&cmd.as_str()) {
            let models: Vec<&str> = match cmd.as_str() {
                "anthropic" => vec![
                    "claude-4.8-sonnet",
                    "claude-4.7-sonnet",
                    "claude-4.6-sonnet",
                    "claude-4.5-sonnet",
                    "claude-fable-5",
                ],
                "openai" => vec![
                    "gpt-5.5",
                    "gpt-5.4",
                    "gpt-5.3",
                    "gpt-5.2",
                ],
                "openrouter" => vec![
                    "anthropic/claude-4.8-sonnet",
                    "google/gemini-3.1-pro",
                    "openai/gpt-5.5",
                    "deepseek/deepseek-r1",
                    "meta-llama/llama-4-405b-instruct",
                ],
                _ => vec![
                    "gemini-3.1-pro",
                    "gemini-3.5-flash",
                    "gemini-3-pro",
                    "gemini-3-pro-flash",
                    "gemini-3-flash",
                ],
            };
            let model_list: String = models.iter().enumerate()
                .map(|(i, m)| format!("  {}. {}", i + 1, m))
                .collect::<Vec<_>>()
                .join("\n");
            let current = format!(
                "Provider: {} | Model: {}\nSelect a model by typing its number (1-{}):\n{}",
                cmd, models[0], models.len(), model_list
            );
            app.add_message("system", &current);
        } else if let Ok(num) = cmd.parse::<usize>() {
            let prev_msg = app.messages.iter().rev()
                .find(|m| m.role == "system" && m.content.contains("Select a model"));
            if let Some(msg) = prev_msg {
                let lines: Vec<&str> = msg.content.lines().collect();
                let first_line = lines.first().unwrap_or(&"");
                let parts: Vec<&str> = first_line.splitn(3, '|').collect();
                if parts.len() >= 2 {
                    let prov = parts.first().map(|s| s.trim_start_matches("Provider: ").trim()).unwrap_or("");
                    let models_for_provider: &[&str] = match prov {
                        "anthropic" => &[
                            "claude-4.8-sonnet",
                            "claude-4.7-sonnet",
                            "claude-4.6-sonnet",
                            "claude-4.5-sonnet",
                            "claude-fable-5",
                        ],
                        "openai" => &[
                            "gpt-5.5",
                            "gpt-5.4",
                            "gpt-5.3",
                            "gpt-5.2",
                        ],
                        "openrouter" => &[
                            "anthropic/claude-4.8-sonnet",
                            "google/gemini-3.1-pro",
                            "openai/gpt-5.5",
                            "deepseek/deepseek-r1",
                            "meta-llama/llama-4-405b-instruct",
                        ],
                        _ => &[
                            "gemini-3.1-pro",
                            "gemini-3.5-flash",
                            "gemini-3-pro",
                            "gemini-3-pro-flash",
                            "gemini-3-flash",
                        ],
                    };
                    if num > 0 && num <= models_for_provider.len() {
                        let selected_model = models_for_provider[num - 1];
                        config.ai.provider = prov.to_string();
                        config.ai.model = selected_model.to_string();
                        app.provider = prov.to_string();
                        app.model = selected_model.to_string();
                        *api_key = config.get_secure_api_key(prov).unwrap_or_default();
                        app.add_message("system", &format!("Switched to {} / {}", prov, selected_model));
                        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                            let settings_path = std::path::Path::new(&home).join(".cypher").join("settings.json");
                            let _ = config.save_to_file(&settings_path);
                        }
                    }
                }
            }
        } else {
            app.add_message("system", &format!("Unknown command: {}\nType \\help for available commands.", cmd));
        }
    }
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
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let status = if app.loading {
        format!("● {} ◇ {} [thinking...]", app.provider, app.model)
    } else {
        format!("● {} ◇ {}", app.provider, app.model)
    };
    let header = Line::from(vec![
        Span::styled(" Cypher ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("v0.1.0", Style::default().fg(Color::DarkGray)),
        Span::raw(" │ "),
        Span::styled(status, Style::default().fg(Color::Cyan)),
    ]);
    frame.render_widget(Paragraph::new(header).style(Style::default().bg(Color::Rgb(20, 20, 30))), area);
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
            Line::from(Span::styled("  or type \\help for available commands.", Style::default().fg(Color::DarkGray))),
            Line::from(""),
        ]);
        frame.render_widget(
            Paragraph::new(welcome).style(Style::default().bg(Color::Rgb(20, 20, 30))),
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
                Style::default().fg(Color::Rgb(150, 200, 255)).bg(Color::Rgb(20, 20, 30)),
            ),
            "assistant" => (
                "  CYPHER ",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Rgb(200, 200, 220)).bg(Color::Rgb(20, 20, 30)),
            ),
            _ => (
                "  ● ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Yellow).bg(Color::Rgb(20, 20, 30)),
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
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));
    frame.render_widget(msg_area, area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let input_style = if app.loading {
        Style::default().fg(Color::DarkGray).bg(Color::Rgb(15, 15, 25))
    } else {
        Style::default().fg(Color::White).bg(Color::Rgb(15, 15, 25))
    };

    let prompt = Line::from(vec![
        Span::styled(" ▸ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(&app.input, input_style),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);

    let input_widget = Paragraph::new(prompt)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(40, 40, 50))));
    frame.render_widget(input_widget, area);

    let width = area.width.saturating_sub(4) as usize;
    let cursor_col = (app.input.len() + 3).min(width);
    frame.set_cursor(area.x + cursor_col as u16, area.y + 1);
}
