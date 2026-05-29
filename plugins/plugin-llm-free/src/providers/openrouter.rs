//! OpenRouter — Free-Tier-Modelle (`:free`-Suffix).
//!
//! Kostenlos nutzbare Modelle mit `:free`-Suffix — echte $0-Kosten.
//! Kein Kreditkarte erforderlich, nur ein kostenloses Konto.
//!
//! Setup: https://openrouter.ai/keys
//! Ported von github.com/apmantza/pi-free (MIT)

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Bekannte freie OpenRouter-Modelle (Stand: Mai 2026, `:free`-Suffix).
/// Diese Modelle kosten $0 — keine Credits werden verbraucht.
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "meta-llama/llama-3.3-70b-instruct:free",
        name:           "Llama 3.3 70B Instruct (free)",
        context_window: 131_072,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "deepseek/deepseek-r1:free",
        name:           "DeepSeek R1 (free)",
        context_window: 164_000,
        max_tokens:     8_000,
        reasoning:      true,
    },
    FreeModel {
        id:             "deepseek/deepseek-chat-v3-0324:free",
        name:           "DeepSeek V3 0324 (free)",
        context_window: 163_840,
        max_tokens:     8_000,
        reasoning:      false,
    },
    FreeModel {
        id:             "qwen/qwen3-14b:free",
        name:           "Qwen3 14B (free)",
        context_window: 40_960,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "google/gemma-3-27b-it:free",
        name:           "Gemma 3 27B IT (free)",
        context_window: 131_072,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "mistralai/mistral-7b-instruct:free",
        name:           "Mistral 7B Instruct (free)",
        context_window: 32_768,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "microsoft/phi-3-mini-128k-instruct:free",
        name:           "Phi-3 Mini 128k (free)",
        context_window: 128_000,
        max_tokens:     4_096,
        reasoning:      false,
    },
];

/// Standard-Modell für direkte Anfragen (bestes kostenloses Coding-Modell).
pub const DEFAULT_MODEL: &str = "meta-llama/llama-3.3-70b-instruct:free";

/// Gibt ProviderConfig zurück, wenn OPENROUTER_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("OPENROUTER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "OpenRouter",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
