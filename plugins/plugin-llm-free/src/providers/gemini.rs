//! Google AI Studio — Gemini Free Tier (sehr großzügig, kein CC).
//!
//! Google AI Studio bietet einen OpenAI-kompatiblen Endpunkt mit
//! großzügigem Free-Tier (keine Kreditkarte erforderlich).
//!
//! Free-Tier-Limits (Stand: Mai 2026):
//!   gemini-2.0-flash-exp: 1.500 Req/Tag, 15 Req/Min, 1M Tokens/Min
//!   gemini-1.5-flash:     1.500 Req/Tag, 15 Req/Min
//!   gemini-1.5-pro:          50 Req/Tag,  2 Req/Min
//!
//! Setup: https://aistudio.google.com/apikey (kostenloser Google-Account)
//!
//! OpenAI-kompatibler Endpunkt:
//!   https://generativelanguage.googleapis.com/v1beta/openai/

use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

/// Bekannte kostenlose Gemini-Modelle (Stand: Mai 2026).
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "gemini-2.0-flash-exp",
        name:           "Gemini 2.0 Flash (free)",
        context_window: 1_048_576,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "gemini-2.5-flash-preview-05-20",
        name:           "Gemini 2.5 Flash Preview",
        context_window: 1_048_576,
        max_tokens:     8_192,
        reasoning:      true,
    },
    FreeModel {
        id:             "gemini-1.5-flash",
        name:           "Gemini 1.5 Flash",
        context_window: 1_048_576,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "gemini-1.5-pro",
        name:           "Gemini 1.5 Pro (50 Req/Tag)",
        context_window: 2_097_152,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell: schnell + großes Kontextfenster + gutes Free-Tier-Limit.
pub const DEFAULT_MODEL: &str = "gemini-2.0-flash-exp";

/// Gibt ProviderConfig zurück, wenn GOOGLE_AI_STUDIO_KEY gesetzt ist.
pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("GOOGLE_AI_STUDIO_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig {
        name:     "Google AI Studio",
        base_url: BASE_URL,
        api_key:  Some(key),
        model:    DEFAULT_MODEL,
    })
}
