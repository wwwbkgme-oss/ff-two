//! LLM-Driver-Modul — implementiert `LlmDriver` aus `domain/agents`.
//!
//! `FreeProviderDriver` ist die produktionsreife Implementierung:
//! probiert 8 kostenlose Provider in Failover-Reihenfolge durch.

pub mod client;
pub mod providers;
pub mod router;
pub mod types;

use async_trait::async_trait;
use agents::{LlmDriver, LlmResponse};
use errors::{AppError, AppResult};

// ── FreeProviderDriver ────────────────────────────────────────────────────────

/// Produktionsreife `LlmDriver`-Implementierung.
///
/// Probiert alle konfigurierten freien Provider in Prioritätsreihenfolge
/// (OpenRouter → Groq → Cerebras → SambaNova → Mistral → Gemini → LLM7 → Ollama).
/// Fällt zurück auf deterministischen Mock wenn alle Provider nicht erreichbar.
///
/// # Aktivierung
/// Mindestens einen ENV-Key setzen (z. B. `GROQ_API_KEY=gsk_...`)
/// oder Ollama lokal starten.
pub struct FreeProviderDriver;

impl FreeProviderDriver {
    pub fn new() -> Self { Self }
}

impl Default for FreeProviderDriver {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl LlmDriver for FreeProviderDriver {
    async fn chat(&self, system: &str, user: &str, max_tokens: u32) -> AppResult<LlmResponse> {
        router::chat(system, user, max_tokens)
            .await
            .map(|r| LlmResponse {
                text:     r.text,
                provider: r.provider.to_owned(),
                model:    r.model,
                tokens:   r.tokens,
            })
            .map_err(|e| AppError::Agent(e.to_string()))
    }

    fn provider_name(&self) -> &str { "free-provider-driver" }
}
