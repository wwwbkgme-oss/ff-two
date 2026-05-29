//! SambaNova — Forever-Free Tier (kein CC, kein Ablauf).
//!
//! Free Tier: 20–480 RPM, 400–9600 RPD — dauerhaft kostenlos.
//! Stärkstes dauerhaft kostenloses Tier aller Cloud-Provider.
//! Läuft auf proprietärer RDU-Hardware.
//!
//! Setup: https://cloud.sambanova.ai
//! Ported von github.com/apmantza/pi-free (MIT)

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.sambanova.ai/v1";

/// Bekannte SambaNova-Modelle — alle dauerhaft kostenlos.
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "Meta-Llama-3.3-70B-Instruct",
        name:           "Llama 3.3 70B Instruct",
        context_window: 131_072,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "Meta-Llama-3.1-405B-Instruct",
        name:           "Llama 3.1 405B Instruct",
        context_window: 131_072,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "Meta-Llama-3.1-8B-Instruct",
        name:           "Llama 3.1 8B Instruct",
        context_window: 131_072,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "DeepSeek-R1",
        name:           "DeepSeek R1",
        context_window: 163_840,
        max_tokens:     16_000,
        reasoning:      true,
    },
    FreeModel {
        id:             "DeepSeek-V3-0324",
        name:           "DeepSeek V3 0324",
        context_window: 163_840,
        max_tokens:     16_000,
        reasoning:      false,
    },
    FreeModel {
        id:             "Qwen2.5-Coder-32B-Instruct",
        name:           "Qwen 2.5 Coder 32B",
        context_window: 32_768,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "Llama-4-Scout-17B-16E-Instruct",
        name:           "Llama 4 Scout 17B",
        context_window: 131_072,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell: bestes Coding-Modell im kostenlosen SambaNova-Tier.
pub const DEFAULT_MODEL: &str = "Qwen2.5-Coder-32B-Instruct";

/// Gibt ProviderConfig zurück, wenn SAMBANOVA_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("SAMBANOVA_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "SambaNova",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
