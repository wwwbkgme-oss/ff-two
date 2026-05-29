//! Failover-Router — probiert Provider in Prioritätsreihenfolge.

use reqwest::Client;
use tracing::{debug, info, warn};

use super::{
    client::{build_client, chat_completions, list_models},
    providers,
    types::{ChatMessage, ChatRequest, DriverCallResult, LlmError, LlmResult, ProviderConfig},
};

pub async fn chat(system: &str, user: &str, max_tokens: u32) -> LlmResult<DriverCallResult> {
    let providers = providers::available_providers();
    if providers.is_empty() { return Err(LlmError::NoProviders); }

    let client = build_client()?;
    let mut last = String::new();

    for p in &providers {
        match try_provider(&client, p, system, user, max_tokens).await {
            Ok(r) => {
                info!(provider = p.name, model = %r.model, "drivers/llm: ok");
                return Ok(r);
            }
            Err(e) => {
                warn!(provider = p.name, error = %e, "drivers/llm: fehlgeschlagen");
                last = format!("{}: {e}", p.name);
            }
        }
    }
    Err(LlmError::AllFailed(last))
}

async fn try_provider(
    client: &Client,
    p:      &ProviderConfig,
    system: &str,
    user:   &str,
    max:    u32,
) -> LlmResult<DriverCallResult> {
    if p.name.starts_with("Ollama") && !ollama_reachable(client, p.base_url).await {
        return Err(LlmError::Provider {
            provider: p.name.to_owned(),
            message:  "nicht erreichbar (läuft `ollama serve`?)".into(),
        });
    }

    let model = resolve_model(client, p).await;
    let req = ChatRequest::new(
        &model,
        vec![ChatMessage::system(system), ChatMessage::user(user)],
        max,
    );
    let resp = chat_completions(client, p.base_url, p.api_key.as_deref(), &req).await?;

    if resp.choices.is_empty() {
        return Err(LlmError::Provider { provider: p.name.to_owned(), message: "leere Antwort".into() });
    }

    Ok(DriverCallResult {
        provider: p.name,
        model:    resp.model.unwrap_or(model),
        text:     resp.text().to_owned(),
        tokens:   resp.usage.as_ref().and_then(|u| u.total_tokens),
    })
}

async fn resolve_model(client: &Client, p: &ProviderConfig) -> String {
    if p.name.starts_with("Ollama") {
        let available = list_models(client, p.base_url, None).await;
        for pref in &["qwen2.5-coder", "deepseek-coder", "codellama", "llama3", "llama2"] {
            if let Some(m) = available.iter().find(|m| m.contains(pref)) {
                debug!(model = %m, "Ollama: Modell gewählt");
                return m.clone();
            }
        }
    }
    p.model.to_owned()
}

async fn ollama_reachable(client: &Client, base_url: &str) -> bool {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    client.head(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send().await
        .map(|r| r.status().is_success() || r.status().as_u16() == 405)
        .unwrap_or(false)
}
