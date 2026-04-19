use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};

use crate::middleware::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub ollama_url: Option<String>,
    pub custom_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

pub struct LlmService {
    client: Client,
}

impl LlmService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn stream_chat(
        &self,
        config: &LlmConfig,
        request: &ChatRequest,
    ) -> Result<reqwest::Response, AppError> {
        let provider_config = self.get_provider_config(config)?;
        
        let body = self.build_request_body(config, &provider_config, request);
        
        let response = self.client
            .post(&provider_config.url)
            .headers(provider_config.headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send LLM request: {}", e);
                AppError::BadRequest(format!("LLM request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("LLM API error: {} - {}", status, body);
            return Err(AppError::BadRequest(format!(
                "LLM API error: {} - {}",
                status, body
            )));
        }

        Ok(response)
    }

    fn get_provider_config(&self, config: &LlmConfig) -> Result<ProviderConfig, AppError> {
        match config.provider.as_str() {
            "openai" => Ok(ProviderConfig {
                url: "https://api.openai.com/v1/chat/completions".to_string(),
                headers: self.build_headers(&[
                    ("Content-Type", "application/json"),
                    ("Authorization", &format!("Bearer {}", config.api_key.as_deref().unwrap_or(""))),
                ]),
            }),
            "anthropic" => Ok(ProviderConfig {
                url: "https://api.anthropic.com/v1/messages".to_string(),
                headers: self.build_headers(&[
                    ("Content-Type", "application/json"),
                    ("x-api-key", config.api_key.as_deref().unwrap_or("")),
                    ("anthropic-version", "2023-06-01"),
                ]),
            }),
            "google" => {
                let api_key = config.api_key.as_deref().unwrap_or("");
                Ok(ProviderConfig {
                    url: format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse",
                        config.model
                    ),
                    headers: self.build_headers(&[
                        ("Content-Type", "application/json"),
                        ("x-goog-api-key", api_key),
                    ]),
                })
            }
            "ollama" => {
                let base_url = config.ollama_url.as_deref().unwrap_or("http://localhost:11434");
                Ok(ProviderConfig {
                    url: format!("{}/v1/chat/completions", base_url),
                    headers: self.build_headers(&[
                        ("Content-Type", "application/json"),
                    ]),
                })
            }
            "minimax" => Ok(ProviderConfig {
                url: "https://api.minimax.io/v1/chat/completions".to_string(),
                headers: self.build_headers(&[
                    ("Content-Type", "application/json"),
                    ("Authorization", &format!("Bearer {}", config.api_key.as_deref().unwrap_or(""))),
                ]),
            }),
            "custom" => {
                let endpoint = config.custom_endpoint.as_deref().unwrap_or("");
                Ok(ProviderConfig {
                    url: format!("{}/chat/completions", endpoint),
                    headers: self.build_headers(&[
                        ("Content-Type", "application/json"),
                        ("Authorization", &format!("Bearer {}", config.api_key.as_deref().unwrap_or(""))),
                    ]),
                })
            }
            _ => Err(AppError::BadRequest(format!(
                "Unknown LLM provider: {}",
                config.provider
            ))),
        }
    }

    fn build_request_body(
        &self,
        config: &LlmConfig,
        provider_config: &ProviderConfig,
        request: &ChatRequest,
    ) -> serde_json::Value {
        match config.provider.as_str() {
            "openai" | "ollama" | "minimax" | "custom" => {
                let mut body = serde_json::json!({
                    "model": config.model,
                    "messages": request.messages.iter().map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content
                        })
                    }).collect::<Vec<_>>(),
                    "stream": true,
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] = serde_json::json!(temp);
                }
                if let Some(max_tokens) = request.max_tokens {
                    body["max_tokens"] = serde_json::json!(max_tokens);
                }

                body
            }
            "anthropic" => {
                let system_messages: Vec<&ChatMessage> = request.messages.iter()
                    .filter(|m| m.role == "system")
                    .collect();
                let conversation_messages: Vec<&ChatMessage> = request.messages.iter()
                    .filter(|m| m.role != "system")
                    .collect();

                let system = system_messages.iter()
                    .map(|m| m.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n");

                let mut body = serde_json::json!({
                    "model": config.model,
                    "messages": conversation_messages.iter().map(|m| {
                        serde_json::json!({
                            "role": if m.role == "assistant" { "assistant" } else { "user" },
                            "content": m.content
                        })
                    }).collect::<Vec<_>>(),
                    "stream": true,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if !system.is_empty() {
                    body["system"] = serde_json::json!(system);
                }

                if let Some(temp) = request.temperature {
                    body["temperature"] = serde_json::json!(temp);
                }

                body
            }
            "google" => {
                let system_messages: Vec<&ChatMessage> = request.messages.iter()
                    .filter(|m| m.role == "system")
                    .collect();
                let conversation_messages: Vec<&ChatMessage> = request.messages.iter()
                    .filter(|m| m.role != "system")
                    .collect();

                let contents: Vec<serde_json::Value> = conversation_messages.iter().map(|m| {
                    serde_json::json!({
                        "role": if m.role == "assistant" { "model" } else { "user" },
                        "parts": [{"text": m.content}]
                    })
                }).collect();

                let mut body = serde_json::json!({
                    "contents": contents,
                });

                if !system_messages.is_empty() {
                    body["systemInstruction"] = serde_json::json!({
                        "parts": system_messages.iter().map(|m| {
                            serde_json::json!({"text": m.content})
                        }).collect::<Vec<_>>()
                    });
                }

                body
            }
            _ => serde_json::json!({}),
        }
    }

    fn build_headers(&self, pairs: &[(&str, &str)]) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in pairs {
            if !value.is_empty() {
                if let Ok(header_value) = reqwest::header::HeaderValue::from_str(value) {
                    headers.insert(
                        reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        header_value,
                    );
                }
            }
        }
        headers
    }
}

struct ProviderConfig {
    url: String,
    headers: reqwest::header::HeaderMap,
}
