//! Interne Typen für LLM HTTP-Calls (OpenAI-kompatibles Schema).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("Netzwerk: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Kein Provider konfiguriert")]
    NoProviders,
    #[error("Alle Provider fehlgeschlagen: {0}")]
    AllFailed(String),
    #[error("Provider {provider}: {message}")]
    Provider { provider: String, message: String },
}

pub type LlmResult<T> = std::result::Result<T, LlmError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(s: impl Into<String>) -> Self { Self { role: "system".into(), content: s.into() } }
    pub fn user(s: impl Into<String>)   -> Self { Self { role: "user".into(),   content: s.into() } }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model:       String,
    pub messages:    Vec<ChatMessage>,
    pub max_tokens:  u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    pub stream:      bool,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>, max_tokens: u32) -> Self {
        Self { model: model.into(), messages, max_tokens, temperature: Some(0.2), stream: false }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    pub usage:   Option<ChatUsage>,
    pub model:   Option<String>,
}

impl ChatResponse {
    pub fn text(&self) -> &str {
        self.choices.first().map(|c| c.message.content.as_str()).unwrap_or("")
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAiModel { pub id: String }

#[derive(Debug, Deserialize)]
pub struct OpenAiModelsResponse { pub data: Vec<OpenAiModel> }

/// Konfiguration eines einzelnen Providers.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name:     &'static str,
    pub base_url: &'static str,
    pub api_key:  Option<String>,
    pub model:    &'static str,
}

/// Ergebnis eines erfolgreichen Provider-Calls.
#[derive(Debug)]
pub struct DriverCallResult {
    pub provider: &'static str,
    pub model:    String,
    pub text:     String,
    pub tokens:   Option<u32>,
}
