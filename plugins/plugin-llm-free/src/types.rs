//! Shared types für den Free-LLM-Router.
//!
//! Alle Typen sind OpenAI-API-kompatibel (chat/completions Schema).
//! Ported + rebranded from github.com/apmantza/pi-free (MIT).

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Fehler ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no providers available — configure at least one API key or run Ollama locally")]
    NoProviders,

    #[error("all providers failed: {0}")]
    AllFailed(String),

    #[error("provider {provider} error: {message}")]
    Provider { provider: String, message: String },
}

pub type LlmResult<T> = std::result::Result<T, LlmError>;

// ── Eingabe-Typen (OpenAI-kompatibel) ────────────────────────────────────────

/// Eine einzelne Nachricht im Chat-Format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
}

/// Chat-Completion-Request (OpenAI-kompatibel).
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
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model:       model.into(),
            messages,
            max_tokens:  4096,
            temperature: Some(0.2),
            stream:      false,
        }
    }
}

// ── Ausgabe-Typen (OpenAI-kompatibel) ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens:     Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens:      Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
    pub usage:   Option<ChatUsage>,
    pub model:   Option<String>,
}

impl ChatResponse {
    /// Gibt den Text der ersten Antwortnachricht zurück.
    pub fn text(&self) -> &str {
        self.choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("")
    }
}

// ── OpenAI Models-Endpoint ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OpenAiModel {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiModelsResponse {
    pub data: Vec<OpenAiModel>,
}

// ── Modell-Metadaten ──────────────────────────────────────────────────────────

/// Bekannte freie Modelle mit Metadaten für die Auswahl.
#[derive(Debug, Clone)]
pub struct FreeModel {
    /// Provider-interner Modell-ID (z. B. `meta-llama/llama-3.3-70b-instruct:free`)
    pub id:             &'static str,
    /// Lesbarer Name
    pub name:           &'static str,
    /// Kontext-Fenster in Tokens
    pub context_window: u32,
    /// Max. Output-Tokens
    pub max_tokens:     u32,
    /// Reasoning-Modell? (z. B. DeepSeek R1)
    pub reasoning:      bool,
}

// ── Provider-Konfiguration ────────────────────────────────────────────────────

/// Vollständige Konfiguration für einen Provider-Aufruf.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Lesbarer Name (z. B. "OpenRouter")
    pub name:     &'static str,
    /// Base-URL der OpenAI-kompatiblen API
    pub base_url: &'static str,
    /// API-Key (falls nötig)
    pub api_key:  Option<String>,
    /// Standard-Modell für diesen Provider
    pub model:    &'static str,
}

/// Ergebnis eines Provider-Aufrufs inkl. Provenance.
#[derive(Debug)]
pub struct LlmCallResult {
    pub provider: &'static str,
    pub model:    String,
    pub text:     String,
    pub tokens:   Option<u32>,
}
