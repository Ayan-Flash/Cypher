use crate::error::{CypherError, Result};
use futures::StreamExt;

const SYSTEM_PROMPT: &str = "You are the Cypher Security AI Assistant. You are a world-class cybersecurity expert and secure developer. \
     You must focus exclusively on software security, web security, cryptography, DevSecOps, vulnerabilities, secure coding, and threat modeling. \
     If the user asks a general programming, architectural, or technical question, do NOT refuse it outright. Instead, you must analyze and answer it specifically from a security angle \
     (e.g., discuss security implications, inputs/bounds validation, threat scenarios, resource exhaustion, or secure coding best practices related to their question). \
     Only if the user's request is completely non-technical, non-software-related, or cannot possibly be framed from a security perspective should you politely decline \
     and direct them back to cybersecurity topics.";

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
    match provider {
        "anthropic" => stream_anthropic(client, model, api_key, prompt, on_chunk).await,
        "openrouter" => {
            stream_openai_compatible(client, "https://openrouter.ai/api/v1/chat/completions", model, api_key, prompt, on_chunk).await
        }
        "openai" => {
            stream_openai_compatible(client, "https://api.openai.com/v1/chat/completions", model, api_key, prompt, on_chunk).await
        }
        _ => stream_gemini(client, model, api_key, prompt, on_chunk).await,
    }
}

async fn stream_openai_compatible(
    client: &reqwest::Client,
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

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| CypherError::Scanner(format!("Failed to connect: {}", e)))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(CypherError::Scanner(format!("API error: {}", err_text)));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| CypherError::Scanner(format!("Stream error: {}", e)))?;
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

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| CypherError::Scanner(format!("Failed to connect to Anthropic: {}", e)))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(CypherError::Scanner(format!("Anthropic API error: {}", err_text)));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| CypherError::Scanner(format!("Stream error: {}", e)))?;
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

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| CypherError::Scanner(format!("Failed to connect to Gemini: {}", e)))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(CypherError::Scanner(format!("Gemini API error: {}", err_text)));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| CypherError::Scanner(format!("Stream error: {}", e)))?;
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
