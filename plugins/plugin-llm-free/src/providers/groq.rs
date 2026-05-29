//! Groq — Großzügiges kostenloses Tier (ultra-schnelle LPU-Inferenz).
//!
//! Free Tier: 14 400 Req/Tag, 100 Req/Min — kein CC, kein Ablauf.
//! Bekannt für niedrigste Latenz aller Cloud-LLM-Provider.
//!
//! Setup: https://console.groq.com
//! Ported von github.com/apmantza/pi-free (MIT)

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// Bekannte kostenlose Groq-Modelle (Stand: Mai 2026).
/// Alle Groq-Modelle haben ein dauerhaft kostenloses Tier.
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "llama-3.3-70b-versatile",
        name:           "Llama 3.3 70B Versatile",
        context_window: 128_000,
        max_tokens:     32_768,
        reasoning:      false,
    },
    FreeModel {
        id:             "llama-3.1-8b-instant",
        name:           "Llama 3.1 8B Instant",
        context_window: 128_000,
        max_tokens:     8_000,
        reasoning:      false,
    },
    FreeModel {
        id:             "deepseek-r1-distill-llama-70b",
        name:           "DeepSeek R1 Distill Llama 70B",
        context_window: 128_000,
        max_tokens:     16_000,
        reasoning:      true,
    },
    FreeModel {
        id:             "qwen-qwq-32b",
        name:           "QwQ 32B",
        context_window: 32_768,
        max_tokens:     16_000,
        reasoning:      true,
    },
    FreeModel {
        id:             "gemma2-9b-it",
        name:           "Gemma 2 9B IT",
        context_window: 8_192,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "llama3-70b-8192",
        name:           "Llama 3 70B 8192",
        context_window: 8_192,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell: beste Balance aus Qualität + Geschwindigkeit.
pub const DEFAULT_MODEL: &str = "llama-3.3-70b-versatile";

/// Gibt ProviderConfig zurück, wenn GROQ_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("GROQ_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "Groq",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
