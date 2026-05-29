//! Ollama — Lokal, 100 % kostenlos, kein API-Key erforderlich.
//!
//! Ollama führt Open-Source-Modelle lokal auf dem eigenen Rechner aus.
//! Keinerlei Kosten, keine Rate-Limits, keine Registrierung nötig.
//! Erfordert: https://ollama.com installieren + mindestens ein Modell pullen.
//!
//! Standard-Endpunkt: http://localhost:11434/v1
//! Konfigurierbar via ENV OLLAMA_BASE_URL (z. B. für Remote-Ollama-Instanz).
//!
//! Empfohlene lokale Modelle für Coding-Tasks:
//!   ollama pull codellama:7b
//!   ollama pull deepseek-coder:6.7b
//!   ollama pull qwen2.5-coder:7b
//!   ollama pull llama3.1:8b

use crate::types::{FreeModel, ProviderConfig};

/// Default-Base-URL — überschreibbar via OLLAMA_BASE_URL.
pub const DEFAULT_BASE_URL: &str = "http://localhost:11434/v1";

/// Empfohlene lokale Coding-Modelle (müssen vorher gepullt werden).
/// Nur zu Dokumentationszwecken — tatsächlich verfügbare Modelle werden
/// zur Laufzeit via /models-Endpunkt entdeckt.
pub const RECOMMENDED_MODELS: &[FreeModel] = &[
    FreeModel {
        id:             "qwen2.5-coder:7b",
        name:           "Qwen 2.5 Coder 7B (lokal)",
        context_window: 32_768,
        max_tokens:     8_192,
        reasoning:      false,
    },
    FreeModel {
        id:             "deepseek-coder:6.7b",
        name:           "DeepSeek Coder 6.7B (lokal)",
        context_window: 16_384,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "codellama:7b",
        name:           "Code Llama 7B (lokal)",
        context_window: 16_384,
        max_tokens:     4_096,
        reasoning:      false,
    },
    FreeModel {
        id:             "llama3.1:8b",
        name:           "Llama 3.1 8B (lokal)",
        context_window: 128_000,
        max_tokens:     8_192,
        reasoning:      false,
    },
];

/// Standard-Modell für Coding (bestes Preis-Leistungs-Verhältnis lokal).
pub const DEFAULT_MODEL: &str = "qwen2.5-coder:7b";

/// Gibt immer eine ProviderConfig zurück — Ollama ist kein Key erforderlich.
/// Base-URL ist konfigurierbar via `OLLAMA_BASE_URL`.
///
/// Hinweis: Der Router prüft, ob Ollama tatsächlich erreichbar ist, bevor er
/// ihn nutzt (im `router.rs` Failover-Mechanismus).
pub fn config() -> Option<ProviderConfig> {
    let base_url: &'static str = {
        // Lese ENV zur Laufzeit und leak den String für 'static lifetime.
        // Notwendig weil ProviderConfig `base_url: &'static str` erwartet.
        // Alternative wäre String in ProviderConfig — bleibt für jetzt simpel.
        match std::env::var("OLLAMA_BASE_URL") {
            Ok(url) if !url.is_empty() => {
                // SAFETY: der geleakte String lebt für die Prozesslebensdauer.
                // Ollama-Config wird einmalig beim Start gelesen.
                Box::leak(url.into_boxed_str())
            }
            _ => DEFAULT_BASE_URL,
        }
    };

    Some(ProviderConfig {
        name:     "Ollama (local)",
        base_url,
        api_key:  None, // kein Key nötig
        model:    DEFAULT_MODEL,
    })
}
