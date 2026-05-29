//! Generischer async OpenAI-kompatibler HTTP-Client.

use reqwest::Client;
use tracing::debug;

use super::types::{ChatRequest, ChatResponse, LlmError, LlmResult, OpenAiModelsResponse};

const TIMEOUT_SECS: u64 = 60;

pub fn build_client() -> LlmResult<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(LlmError::Network)
}

pub async fn chat_completions(
    client:   &Client,
    base_url: &str,
    api_key:  Option<&str>,
    req:      &ChatRequest,
) -> LlmResult<ChatResponse> {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    debug!(provider = %base_url, model = %req.model, "chat_completions");

    let mut b = client.post(&url).json(req);
    if let Some(k) = api_key {
        if !k.is_empty() { b = b.bearer_auth(k); }
    }

    let resp   = b.send().await.map_err(LlmError::Network)?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(LlmError::Http { status: status.as_u16(), body });
    }
    resp.json::<ChatResponse>().await.map_err(LlmError::Network)
}

pub async fn list_models(
    client:   &Client,
    base_url: &str,
    api_key:  Option<&str>,
) -> Vec<String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let mut b = client.get(&url);
    if let Some(k) = api_key { if !k.is_empty() { b = b.bearer_auth(k); } }
    match b.send().await {
        Ok(r) if r.status().is_success() =>
            r.json::<OpenAiModelsResponse>().await
             .map(|m| m.data.into_iter().map(|m| m.id).collect())
             .unwrap_or_default(),
        _ => vec![],
    }
}
