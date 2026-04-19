use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::post,
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};

use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::middleware::permission::check_project_permission;
use crate::services::llm::{LlmConfig, ChatMessage as LlmChatMessage, LlmService};

#[derive(Deserialize)]
pub struct ApiChatRequest {
    pub project_id: Option<uuid::Uuid>,
    pub messages: Vec<LlmChatMessage>,
    pub config: LlmConfig,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub content: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stream", post(stream_chat))
        .route("/chat", post(chat))
}

pub async fn stream_chat(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<ApiChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    if let Some(project_id) = req.project_id {
        if !check_project_permission(&state.db, claims.0.sub, project_id, "chat:use").await? {
            return Err(AppError::PermissionDenied);
        }
    }

    let llm_service = LlmService::new();
    
    let chat_req = crate::services::llm::ChatRequest {
        messages: req.messages.clone(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };

    let response = llm_service.stream_chat(&req.config, &chat_req).await?;

    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        let bytes_stream = response.bytes_stream();
        use futures::StreamExt;
        
        let mut buffer = String::new();
        
        let mut stream = bytes_stream;
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    let lines = text.split('\n');
                    
                    for line in lines {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        let token = parse_sse_line(trimmed, &req.config.provider);
                        if let Some(token_text) = token {
                            if let Err(_) = tx.send(Ok(Event::default().data(&token_text))).await {
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading stream: {}", e);
                    let _ = tx.send(Ok(Event::default().data(&format!("[ERROR] {}", e)))).await;
                    break;
                }
            }
        }

        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
    });

    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    ))
}

pub async fn chat(
    State(state): State<AppState>,
    claims: axum::extract::Extension<Claims>,
    Json(req): Json<ApiChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    if let Some(project_id) = req.project_id {
        if !check_project_permission(&state.db, claims.0.sub, project_id, "chat:use").await? {
            return Err(AppError::PermissionDenied);
        }
    }

    let llm_service = LlmService::new();
    
    let chat_req = crate::services::llm::ChatRequest {
        messages: req.messages.clone(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };

    let response = llm_service.stream_chat(&req.config, &chat_req).await?;

    let bytes = response.bytes().await.map_err(|e| {
        error!("Failed to read response bytes: {}", e);
        AppError::Internal
    })?;

    let text = String::from_utf8_lossy(&bytes);
    let full_content = parse_full_response(&text, &req.config.provider);

    Ok(Json(ChatResponse {
        content: full_content,
    }))
}

fn parse_sse_line(line: &str, provider: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }
    
    let data = line[6..].trim();
    
    if data == "[DONE]" {
        return None;
    }

    match provider {
        "openai" | "ollama" | "minimax" | "custom" => parse_openai_line(data),
        "anthropic" => parse_anthropic_line(data),
        "google" => parse_google_line(data),
        _ => None,
    }
}

fn parse_openai_line(data: &str) -> Option<String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(choices) = parsed.get("choices") {
            if let Some(choice) = choices.as_array()?.first() {
                if let Some(delta) = choice.get("delta") {
                    if let Some(content) = delta.get("content") {
                        return content.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
    }
    None
}

fn parse_anthropic_line(data: &str) -> Option<String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(event_type) = parsed.get("type").and_then(|v| v.as_str()) {
            if event_type == "content_block_delta" {
                if let Some(delta) = parsed.get("delta") {
                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
        }
    }
    None
}

fn parse_google_line(data: &str) -> Option<String> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
        if let Some(candidates) = parsed.get("candidates") {
            if let Some(candidate) = candidates.as_array()?.first() {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts") {
                        if let Some(part) = parts.as_array()?.first() {
                            if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                                return Some(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_full_response(response_text: &str, provider: &str) -> String {
    let mut content = String::new();
    
    for line in response_text.split('\n') {
        let trimmed = line.trim();
        if let Some(token) = parse_sse_line(trimmed, provider) {
            content.push_str(&token);
        }
    }
    
    content
}
