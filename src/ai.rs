use crate::error::{CypherError, Result};
use futures::StreamExt;
use std::time::Duration;

const SYSTEM_PROMPT: &str = "You are the Cypher Security AI Assistant. You are a world-class cybersecurity expert and secure developer. \
     You must focus exclusively on software security, web security, cryptography, DevSecOps, vulnerabilities, secure coding, and threat modeling. \
     If the user asks a general programming, architectural, or technical question, do NOT refuse it outright. Instead, you must analyze and answer it specifically from a security angle \
     (e.g., discuss security implications, inputs/bounds validation, threat scenarios, resource exhaustion, or secure coding best practices related to their question). \
     Only if the user's request is completely non-technical, non-software-related, or cannot possibly be framed from a security perspective should you politely decline \
     and direct them back to cybersecurity topics.";

const MAX_CONNECT_RETRIES: u32 = 2;

/// Build a reqwest client with sane timeouts for all AI provider calls.
pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default()
}

fn provider_label(provider: &str) -> String {
    match provider {
        "anthropic" => "Anthropic".to_string(),
        "openai" => "OpenAI".to_string(),
        "openrouter" => "OpenRouter".to_string(),
        "gemini" => "Gemini".to_string(),
        other => other.to_string(),
    }
}

/// Translate a raw HTTP failure from a provider into a clear, actionable message.
/// The raw body is never shown to the user, only logged at debug level.
fn friendly_provider_error(provider: &str, status: reqwest::StatusCode, body: &str) -> String {
    tracing::debug!("{} API error ({}): {}", provider, status, body);
    let label = provider_label(provider);

    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        format!(
            "{label} rejected your API key (it may be invalid, expired, or missing permissions). Run /models to reconfigure your credentials.",
        )
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        format!("{label} rate limit reached. Please wait a moment and try again.")
    } else if status == reqwest::StatusCode::PAYMENT_REQUIRED {
        format!("{label} reports insufficient credits or a billing issue on this account.")
    } else if status.is_server_error() {
        format!("{label} is temporarily unavailable (server error). Please try again shortly.")
    } else if status == reqwest::StatusCode::NOT_FOUND {
        format!("{label} could not find the requested model. Run /models to pick a different one.")
    } else {
        format!("{label} rejected the request (HTTP {}). Try again, or run /models to check your configuration.", status.as_u16())
    }
}

fn friendly_connect_error(provider: &str, e: &reqwest::Error) -> String {
    tracing::debug!("{} connection error: {}", provider, e);
    let label = provider_label(provider);
    if e.is_timeout() {
        format!("Connecting to {label} timed out. Check your internet connection and try again.")
    } else if e.is_connect() {
        format!("Could not reach {label}. Check your internet connection and try again.")
    } else {
        format!("Failed to communicate with {label}: connection error.")
    }
}

fn should_retry_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_backoff(attempt: u32) -> Duration {
    match attempt {
        1 => Duration::from_millis(300),
        _ => Duration::from_millis(900),
    }
}

#[allow(dead_code)]
pub async fn call_ai_api(
    client: &reqwest::Client,
    provider: &str,
    model: &str,
    api_key: &str,
    prompt: &str,
) -> Result<String> {
    let mut full_response = String::new();
    {
        let mut append = |chunk: &str| {
            full_response.push_str(chunk);
        };
        stream_ai_response(client, provider, model, api_key, prompt, &mut append).await?;
    }
    Ok(full_response)
}

pub async fn stream_ai_response(
    client: &reqwest::Client,
    provider: &str,
    model: &str,
    api_key: &str,
    prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) + Send),
) -> Result<()> {
    if api_key.trim().is_empty() {
        return Err(CypherError::Provider(format!(
            "No API key configured for {}. Run /models to connect a provider.",
            provider_label(provider)
        )));
    }

    match provider {
        "anthropic" => stream_anthropic(client, model, api_key, prompt, on_chunk).await,
        "openrouter" => {
            stream_openai_compatible(client, "openrouter", "https://openrouter.ai/api/v1/chat/completions", model, api_key, prompt, on_chunk).await
        }
        "openai" => {
            stream_openai_compatible(client, "openai", "https://api.openai.com/v1/chat/completions", model, api_key, prompt, on_chunk).await
        }
        _ => stream_gemini(client, model, api_key, prompt, on_chunk).await,
    }
}

/// Send the initial request, retrying transient failures (network errors, 429, 5xx) before any
/// response bytes have streamed. Returns the successful response ready for streaming.
async fn send_with_retry(
    provider: &str,
    build_request: impl Fn() -> reqwest::RequestBuilder,
) -> Result<reqwest::Response> {
    let mut attempt = 0;
    loop {
        match build_request().send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response);
                }
                let status = response.status();
                if should_retry_status(status) && attempt < MAX_CONNECT_RETRIES {
                    attempt += 1;
                    tokio::time::sleep(retry_backoff(attempt)).await;
                    continue;
                }
                let body = response.text().await.unwrap_or_default();
                return Err(CypherError::Provider(friendly_provider_error(provider, status, &body)));
            }
            Err(e) => {
                if attempt < MAX_CONNECT_RETRIES {
                    attempt += 1;
                    tokio::time::sleep(retry_backoff(attempt)).await;
                    continue;
                }
                return Err(CypherError::Provider(friendly_connect_error(provider, &e)));
            }
        }
    }
}

async fn stream_openai_compatible(
    client: &reqwest::Client,
    provider: &str,
    url: &str,
    model: &str,
    api_key: &str,
    prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) + Send),
) -> Result<()> {
    let request_body = serde_json::json!({
        "model": model,
        "stream": true,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt}
        ]
    });

    let response = send_with_retry(provider, || {
        client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&request_body)
    })
    .await?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| CypherError::Provider(friendly_connect_error(provider, &e)))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if let Some(text) = parse_sse_data_line(&line) {
                if text == "[DONE]" {
                    return Ok(());
                }
                if let Some(content) = extract_openai_content(&text) {
                    on_chunk(&content);
                }
            }
        }
    }

    Ok(())
}

async fn stream_anthropic(
    client: &reqwest::Client,
    model: &str,
    api_key: &str,
    prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) + Send),
) -> Result<()> {
    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "stream": true,
        "system": SYSTEM_PROMPT,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let response = send_with_retry("anthropic", || {
        client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
    })
    .await?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| CypherError::Provider(friendly_connect_error("anthropic", &e)))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(delta) = val.get("delta") {
                        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                            on_chunk(text);
                        }
                    } else if let Some(content) = val.get("content_block") {
                        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                            on_chunk(text);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn stream_gemini(
    client: &reqwest::Client,
    model: &str,
    api_key: &str,
    prompt: &str,
    on_chunk: &mut (dyn FnMut(&str) + Send),
) -> Result<()> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    let request_body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("System Instruction: {}\n\nUser Question: {}", SYSTEM_PROMPT, prompt)
            }]
        }]
    });

    let response = send_with_retry("gemini", || client.post(&url).json(&request_body)).await?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| CypherError::Provider(friendly_connect_error("gemini", &e)))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if let Some(text) = parse_sse_data_line(&line) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(candidates) = val.get("candidates").and_then(|c| c.as_array()) {
                        if let Some(first) = candidates.first() {
                            if let Some(content) = first.get("content") {
                                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                    if let Some(first_part) = parts.first() {
                                        if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                                            on_chunk(text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_sse_data_line(line: &str) -> Option<String> {
    line.strip_prefix("data: ").map(|data| data.to_string())
}

fn extract_openai_content(data: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(choices) = val.get("choices").and_then(|c| c.as_array()) {
            if let Some(first) = choices.first() {
                if let Some(delta) = first.get("delta") {
                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                        return Some(content.to_string());
                    }
                }
            }
        }
    }
    None
}
