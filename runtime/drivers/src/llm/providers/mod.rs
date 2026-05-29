//! Free LLM Provider-Katalog — runtime/drivers/llm/providers
//!
//! Failover-Priorität:
//!   OpenRouter → Groq → Cerebras → SambaNova → Mistral →
//!   Google AI Studio → LLM7 → Ollama (lokal, immer letzter Fallback)

pub mod cerebras;
pub mod gemini;
pub mod groq;
pub mod llm7;
pub mod mistral;
pub mod ollama;
pub mod openrouter;
pub mod sambanova;

use crate::llm::types::ProviderConfig;

pub fn available_providers() -> Vec<ProviderConfig> {
    let mut v = Vec::new();
    if let Some(p) = openrouter::config() { v.push(p); }
    if let Some(p) = groq::config()       { v.push(p); }
    if let Some(p) = cerebras::config()   { v.push(p); }
    if let Some(p) = sambanova::config()  { v.push(p); }
    if let Some(p) = mistral::config()    { v.push(p); }
    if let Some(p) = gemini::config()     { v.push(p); }
    if let Some(p) = llm7::config()       { v.push(p); }
    if let Some(p) = ollama::config()     { v.push(p); }
    v
}
