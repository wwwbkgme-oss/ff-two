//! Mistral AI — Free Tier (kostenloser Account, kein CC).
//!
//! Mistral bietet ein dauerhaft kostenloses API-Tier für Entwickler.
//! Modelle: mistral-small-latest, open-mistral-7b, open-mixtral-8x7b.
//!
//! Setup: https://console.mistral.ai (kostenloses Konto)
//! Limits: modellabhängig, dokumentiert unter https://docs.mistral.ai

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.mistral.ai/v1";

/// Bekannte freie Mistral-Modelle (Stand: Mai 2026).
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "mistral-small-latest",
        name:           "Mistral Small (latest)",
        context_window: 128_000,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "open-mistral-7b",
        name:           "Mistral 7B",
        context_window: 32_768,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "open-mixtral-8x7b",
        name:           "Mixtral 8×7B",
        context_window: 32_768,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "open-mixtral-8x22b",
        name:           "Mixtral 8×22B",
        context_window: 65_536,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell: beste Balance aus Qualität und Geschwindigkeit.
pub const DEFAULT_MODEL: &str = "mistral-small-latest";

/// Gibt ProviderConfig zurück, wenn MISTRAL_API_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("MISTRAL_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "Mistral",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
