//! `LlmDriver` — abstraktes Interface für KI-Sprachmodell-Calls.
//!
//! ## BKG / ARCHITECTURE.md
//! * **Trait hier** (domain/agents) — kein I/O, pure Schnittstelle
//! * **Implementierungen in** `runtime/drivers/llm/` — I/O, HTTP, API-Keys
//! * **Verdrahtung in** `runtime/server/main.rs` — wires driver into agent
//!
//! Diese Trennung hält die Domain-Schicht frei von Infrastruktur-Details.
//! `FreeLlmAgent` kennt nur diesen Trait — nie einen konkreten HTTP-Client.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use errors::AppResult;

// ── Response ──────────────────────────────────────────────────────────────────

/// Ergebnis eines einzelnen LLM-Aufrufs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Generierter Text (vollständige Antwort).
    pub text:     String,
    /// Provider-Name, der geantwortet hat (z. B. `"Groq"`, `"OpenRouter"`).
    pub provider: String,
    /// Konkret verwendetes Modell (z. B. `"llama-3.3-70b-versatile"`).
    pub model:    String,
    /// Verbrauchte Tokens — falls vom Provider gemeldet.
    pub tokens:   Option<u32>,
}

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Abstraktes LLM-Interface.
///
/// Implementierungen sind ausschließlich in `runtime/drivers/llm/` erlaubt.
/// `domain/agents` sieht nur diesen Trait.
///
/// # Beispiel (Implementierung in runtime/drivers)
/// ```rust,ignore
/// struct FreeProviderDriver { /* provider config */ }
///
/// #[async_trait]
/// impl LlmDriver for FreeProviderDriver {
///     async fn chat(&self, system: &str, user: &str, max_tokens: u32)
///         -> AppResult<LlmResponse>
///     {
///         // HTTP-Call an Provider-API ...
///     }
/// }
/// ```
#[async_trait]
pub trait LlmDriver: Send + Sync + 'static {
    /// Sendet eine Chat-Anfrage und gibt die generierte Antwort zurück.
    ///
    /// * `system`     — System-Prompt (Rollen-Beschreibung)
    /// * `user`       — User-Nachricht (Task-Inhalt)
    /// * `max_tokens` — Maximale Antwortlänge in Tokens
    async fn chat(
        &self,
        system:     &str,
        user:       &str,
        max_tokens: u32,
    ) -> AppResult<LlmResponse>;

    /// Provider-Name für Logging/Telemetrie.
    /// Standard-Impl gibt `"unknown"` zurück.
    fn provider_name(&self) -> &str { "unknown" }
}

// ── Hilfskonstruktor ──────────────────────────────────────────────────────────

/// Null-Driver — gibt immer einen Mock zurück.
/// Nützlich für Tests ohne echten Provider.
pub struct NullDriver {
    pub name: &'static str,
}

impl NullDriver {
    pub fn new() -> Self { Self { name: "null" } }
}

impl Default for NullDriver {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl LlmDriver for NullDriver {
    async fn chat(&self, _system: &str, user: &str, _max: u32) -> AppResult<LlmResponse> {
        Ok(LlmResponse {
            text:     format!("// NullDriver Mock\n// Input: {}\nfn placeholder() {{}}\n",
                              user.chars().take(80).collect::<String>()),
            provider: "null".into(),
            model:    "mock".into(),
            tokens:   Some(0),
        })
    }

    fn provider_name(&self) -> &str { self.name }
}
