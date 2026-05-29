//! Generischer async OpenAI-kompatibler HTTP-Client.
//!
//! Alle freien Provider (OpenRouter, Groq, Cerebras, SambaNova, LLM7, Ollama)
//! sprechen das gleiche OpenAI-Chat-Completions-API. Dieser Client ist
//! provider-agnostisch — nur Base-URL und API-Key variieren.

use reqwest::Client;
use tracing::debug;

use crate::types::{
    ChatRequest, ChatResponse, LlmError, LlmResult, OpenAiModelsResponse,
};

/// Timeout für alle HTTP-Anfragen.
const REQUEST_TIMEOUT_SECS: u64 = 60;

// ── Client-Erstellung ─────────────────────────────────────────────────────────

/// Erstellt einen reqwest-Client mit angemessenem Timeout (rustls, kein System-OpenSSL).
pub fn build_client() -> LlmResult<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(LlmError::Network)
}

// ── Chat Completions ──────────────────────────────────────────────────────────

/// Sendet einen Chat-Completion-Request an einen OpenAI-kompatiblen Endpunkt.
///
/// # Parameter
/// * `client`   — reqwest-Client (kann wiederverwendet werden)
/// * `base_url` — Provider-Base-URL, z. B. `https://api.groq.com/openai/v1`
/// * `api_key`  — Bearer-Token (leer = kein Authorization-Header)
/// * `req`      — Chat-Request
pub async fn chat_completions(
    client:   &Client,
    base_url: &str,
    api_key:  Option<&str>,
    req:      &ChatRequest,
) -> LlmResult<ChatResponse> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    debug!(provider = %base_url, model = %req.model, "chat_completions request");

    let mut builder = client.post(&url).json(req);
    if let Some(key) = api_key {
        if !key.is_empty() {
            builder = builder.bearer_auth(key);
        }
    }

    let resp = builder.send().await.map_err(LlmError::Network)?;
    let status = resp.status();

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(LlmError::Http { status: status.as_u16(), body });
    }

    let response: ChatResponse = resp.json().await.map_err(LlmError::Network)?;
    debug!(provider = %base_url, model = %req.model, "chat_completions ok");
    Ok(response)
}

// ── Models Discovery ──────────────────────────────────────────────────────────

/// Listet verfügbare Modelle von einem OpenAI-kompatiblen `/models`-Endpunkt auf.
/// Gibt leeren Vec zurück bei Fehler (z. B. Provider offline).
pub async fn list_models(
    client:   &Client,
    base_url: &str,
    api_key:  Option<&str>,
) -> Vec<String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let mut builder = client.get(&url);
    if let Some(key) = api_key {
        if !key.is_empty() {
            builder = builder.bearer_auth(key);
        }
    }

    match builder.send().await {
        Err(e) => {
            debug!(base_url, error = %e, "list_models: network error");
            vec![]
        }
        Ok(resp) if !resp.status().is_success() => {
            debug!(base_url, status = %resp.status(), "list_models: non-2xx");
            vec![]
        }
        Ok(resp) => match resp.json::<OpenAiModelsResponse>().await {
            Ok(r) => r.data.into_iter().map(|m| m.id).collect(),
            Err(e) => {
                debug!(base_url, error = %e, "list_models: parse error");
                vec![]
            }
        },
    }
}
