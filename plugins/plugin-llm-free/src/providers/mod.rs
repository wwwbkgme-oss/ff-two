//! Free LLM Provider-Katalog.
//!
//! Nur Anbieter mit **dauerhaft kostenlosem** Tier — keine Kreditkarte,
//! kein auslaufendes Trial-Guthaben.
//!
//! Failover-Priorität:
//!   1. OpenRouter      — beste :free-Modelle (DeepSeek R1, Llama 3.3 70B)
//!   2. Groq            — schnellste Inferenz, 14.400 Req/Tag
//!   3. Cerebras        — CS3-Hardware, 60 Req/Min
//!   4. SambaNova       — forever free, kein CC, große Modelle
//!   5. Mistral         — open-weight Modelle, freier Account
//!   6. Google (Gemini) — 1.500 Req/Tag, OpenAI-kompatibler Endpunkt
//!   7. LLM7            — Multi-Provider-Gateway, 100 Req/Std
//!   8. Ollama          — lokal, 100 % kostenlos, kein Key nötig

pub mod cerebras;
pub mod gemini;
pub mod groq;
pub mod llm7;
pub mod mistral;
pub mod ollama;
pub mod openrouter;
pub mod sambanova;

use crate::types::ProviderConfig;

/// Gibt alle konfigurierten Provider in Failover-Reihenfolge zurück.
/// Nur Provider mit gesetztem API-Key (oder keylosem Zugang) werden inkludiert.
pub fn available_providers() -> Vec<ProviderConfig> {
    let mut out = Vec::new();
    if let Some(p) = openrouter::config() { out.push(p); }
    if let Some(p) = groq::config()       { out.push(p); }
    if let Some(p) = cerebras::config()   { out.push(p); }
    if let Some(p) = sambanova::config()  { out.push(p); }
    if let Some(p) = mistral::config()    { out.push(p); }
    if let Some(p) = gemini::config()     { out.push(p); }
    if let Some(p) = llm7::config()       { out.push(p); }
    if let Some(p) = ollama::config()     { out.push(p); } // immer (lokal)
    out
}
