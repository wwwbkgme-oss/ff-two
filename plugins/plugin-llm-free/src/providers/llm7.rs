//! LLM7.io — Kostenloses Multi-Provider-Gateway.
//!
//! Free Tier: 100 Req/Stunde, 20 Req/Min, 2 Req/Sek — dauerhaft kostenlos.
//! Routet transparent über mehrere Backend-Provider (OpenAI, Mistral, Google,
//! DeepSeek, Cloudflare, u.a.). Der Caller wählt einen Selektor-Alias,
//! nicht ein konkretes Modell.
//!
//! Setup: https://token.llm7.io (kostenloser Token, kein CC)
//! Ported von github.com/apmantza/pi-free (MIT)

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.llm7.io/v1";

/// LLM7-Selektoren — abstrakte Aliasse, kein konkretes Modell.
/// `default` und `fast` sind kostenlos; `pro` erfordert ein Abo.
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "default",
        name:           "LLM7 Default (bestes verfügbares freies Modell)",
        context_window: 32_000,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "fast",
        name:           "LLM7 Fast (niedrigste Latenz)",
        context_window: 32_000,
        max_tokens:     4_096,
        reasoning:      false,
    },
];

/// Standard-Selektor: beste Qualität im kostenlosen Tier.
pub const DEFAULT_MODEL: &str = "default";

/// Gibt ProviderConfig zurück, wenn LLM7_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("LLM7_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "LLM7",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
