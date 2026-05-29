//! Free LLM Provider-Katalog.
//!
//! Nur Anbieter mit **dauerhaft kostenlosem** Tier — keine Kreditkarte,
//! kein auslaufendes Trial-Guthaben.
//!
//! Reihenfolge = Standard-Failover-Priorität:
//!   1. OpenRouter   — beste freie Modelle (DeepSeek R1, Llama 3.3 70B)
//!   2. Groq         — schnellste Inferenz, großzügiges Free-Tier
//!   3. Cerebras     — schnelle RDU-Inferenz, Free-Tier
//!   4. SambaNova    — forever free, kein CC, große Modelle
//!   5. LLM7         — freies Multi-Provider-Gateway, 100 req/hr
//!   6. Ollama       — lokal, 100 % kostenlos, kein Key nötig

pub mod cerebras;
pub mod groq;
pub mod llm7;
pub mod ollama;
pub mod openrouter;
pub mod sambanova;

use crate::types::ProviderConfig;

/// Gibt alle konfigurierten Provider in Failover-Reihenfolge zurück.
/// Nur Provider mit gesetztem API-Key (oder keylosem Zugang) werden inkludiert.
pub fn available_providers() -> Vec<ProviderConfig> {
    let mut out = Vec::new();
    if let Some(p) = openrouter::config()  { out.push(p); }
    if let Some(p) = groq::config()        { out.push(p); }
    if let Some(p) = cerebras::config()    { out.push(p); }
    if let Some(p) = sambanova::config()   { out.push(p); }
    if let Some(p) = llm7::config()        { out.push(p); }
    if let Some(p) = ollama::config()      { out.push(p); } // immer verfügbar (lokal)
    out
}
