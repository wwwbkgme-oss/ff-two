//! Failover-Router für freie LLM-Provider.
//!
//! Probiert Provider in Prioritätsreihenfolge durch. Wenn ein Provider
//! antwortet, wird das Ergebnis sofort zurückgegeben. Nur wenn alle
//! Provider fehlschlagen, wird ein Fehler zurückgegeben.
//!
//! Besonderheit Ollama: vor dem Aufruf wird geprüft, ob der lokale
//! Dienst erreichbar ist (schneller HEAD-Request).

use reqwest::Client;
use tracing::{debug, info, warn};

use crate::{
    client::{build_client, chat_completions, list_models},
    providers,
    types::{ChatMessage, ChatRequest, LlmCallResult, LlmError, LlmResult, ProviderConfig},
};

// ── Öffentliche API ───────────────────────────────────────────────────────────

/// Sendet eine Chat-Anfrage an den ersten verfügbaren freien Provider.
///
/// Provider werden in der von `providers::available_providers()` gelieferten
/// Reihenfolge ausprobiert. Bei Netzwerk- oder HTTP-Fehlern wird auf den
/// nächsten Provider gewechselt. Inhaltliche Fehler (API-Ablehnung o. ä.)
/// werden weitergegeben.
pub async fn chat(
    system:  impl Into<String>,
    user:    impl Into<String>,
    max_tok: u32,
) -> LlmResult<LlmCallResult> {
    let providers = providers::available_providers();

    if providers.is_empty() {
        return Err(LlmError::NoProviders);
    }

    let client  = build_client()?;
    let system  = system.into();
    let user    = user.into();
    let mut last_err = String::new();

    for provider in &providers {
        match try_provider(&client, provider, &system, &user, max_tok).await {
            Ok(result) => {
                info!(
                    provider = provider.name,
                    model    = %result.model,
                    tokens   = ?result.tokens,
                    "free-llm: request succeeded"
                );
                return Ok(result);
            }
            Err(e) => {
                warn!(
                    provider = provider.name,
                    error    = %e,
                    "free-llm: provider failed, trying next"
                );
                last_err = format!("{}: {}", provider.name, e);
            }
        }
    }

    Err(LlmError::AllFailed(last_err))
}

/// Listet alle erreichbaren freien Modelle über alle konfigurierten Provider.
/// Gibt eine flache Liste von `"provider-name/model-id"` zurück.
pub async fn list_all_free_models() -> Vec<String> {
    let providers = providers::available_providers();
    let client    = match build_client() {
        Ok(c)  => c,
        Err(_) => return vec![],
    };

    let mut result = Vec::new();
    for p in &providers {
        // Ollama: dynamisch entdecken
        if p.name.starts_with("Ollama") {
            let models = list_models(&client, p.base_url, p.api_key.as_deref()).await;
            for m in models {
                result.push(format!("ollama/{m}"));
            }
        } else {
            // Für alle anderen: statische Modell-Liste aus dem jeweiligen Modul verwenden
            // (vermeidet unnötige API-Aufrufe)
            let prefix = p.name.to_lowercase().replace(' ', "-");
            result.push(format!("{}/{}", prefix, p.model));
        }
    }
    result
}

// ── Interne Implementierung ───────────────────────────────────────────────────

/// Versucht einen einzelnen Provider. Gibt Fehler zurück ohne Panik.
async fn try_provider(
    client:   &Client,
    provider: &ProviderConfig,
    system:   &str,
    user:     &str,
    max_tok:  u32,
) -> LlmResult<LlmCallResult> {
    // Ollama: Erreichbarkeit vorab prüfen (vermeidet langen Timeout)
    if provider.name.starts_with("Ollama") {
        if !is_ollama_reachable(client, provider.base_url).await {
            return Err(LlmError::Provider {
                provider: provider.name.to_owned(),
                message:  "Ollama ist nicht erreichbar — läuft `ollama serve`?".into(),
            });
        }
    }

    let model = resolve_model(client, provider).await;

    let req = ChatRequest {
        model:       model.clone(),
        messages:    vec![
            ChatMessage::system(system),
            ChatMessage::user(user),
        ],
        max_tokens:  max_tok,
        temperature: Some(0.2),
        stream:      false,
    };

    let resp = chat_completions(
        client,
        provider.base_url,
        provider.api_key.as_deref(),
        &req,
    )
    .await?;

    if resp.choices.is_empty() {
        return Err(LlmError::Provider {
            provider: provider.name.to_owned(),
            message:  "leere Antwort (keine choices)".into(),
        });
    }

    let text   = resp.text().to_owned();
    let tokens = resp.usage.as_ref().and_then(|u| u.total_tokens);
    let model  = resp.model.unwrap_or(model);

    Ok(LlmCallResult { provider: provider.name, model, text, tokens })
}

/// Bestimmt das tatsächlich zu verwendende Modell.
/// Für Ollama: wählt das erste verfügbare lokale Modell.
/// Für alle anderen: verwendet das konfigurierte Standard-Modell.
async fn resolve_model(client: &Client, provider: &ProviderConfig) -> String {
    if provider.name.starts_with("Ollama") {
        let available = list_models(client, provider.base_url, None).await;
        if !available.is_empty() {
            // Bevorzuge bekannte Coding-Modelle, dann erstes verfügbares
            for preferred in &[
                "qwen2.5-coder",
                "deepseek-coder",
                "codellama",
                "llama3",
                "llama2",
            ] {
                if let Some(m) = available.iter().find(|m| m.contains(preferred)) {
                    debug!(model = %m, "Ollama: bevorzugtes Coding-Modell gewählt");
                    return m.clone();
                }
            }
            debug!(model = %available[0], "Ollama: erstes verfügbares Modell gewählt");
            return available[0].clone();
        }
    }
    provider.model.to_owned()
}

/// Prüft, ob der Ollama-Server erreichbar ist (HEAD /v1/models, max 3 s).
async fn is_ollama_reachable(client: &Client, base_url: &str) -> bool {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    match client
        .head(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(r)  => r.status().is_success() || r.status().as_u16() == 405,
        Err(_) => false,
    }
}
