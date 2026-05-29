//! Cerebras — Free Tier auf CS3 Wafer-Scale Hardware.
//!
//! Free Tier: 60 Req/Min, 1M Tokens/Min — dauerhaft kostenlos, kein CC.
//! Beste Inferenzgeschwindigkeit für Llama und DeepSeek.
//!
//! Setup: https://cloud.cerebras.ai
//! Ported von github.com/apmantza/pi-free (MIT)

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.cerebras.ai/v1";

/// Bekannte kostenlose Cerebras-Modelle (Stand: Mai 2026).
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "llama-3.3-70b",
        name:           "Llama 3.3 70B",
        context_window: 128_000,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "llama3.1-8b",
        name:           "Llama 3.1 8B",
        context_window: 128_000,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "llama3.1-70b",
        name:           "Llama 3.1 70B",
        context_window: 128_000,
        max_tokens:     8_192,
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
        id:             "qwen-3-32b",
        name:           "Qwen 3 32B",
        context_window: 32_768,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell: schnellste Inferenz für Coding-Tasks.
pub const DEFAULT_MODEL: &str = "llama-3.3-70b";

/// Gibt ProviderConfig zurück, wenn CEREBRAS_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("CEREBRAS_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "Cerebras",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
