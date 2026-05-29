use crate::llm::types::ProviderConfig;
pub const DEFAULT_BASE: &str = "http://localhost:11434/v1";
pub const DEFAULT_MODEL: &str = "qwen2.5-coder:7b";
/// Ollama ist immer verfügbar (lokal, kein Key nötig).
pub fn config() -> Option<ProviderConfig> {
    let base_url: &'static str = match std::env::var("OLLAMA_BASE_URL") {
        Ok(u) if !u.is_empty() => Box::leak(u.into_boxed_str()),
        _ => DEFAULT_BASE,
    };
    Some(ProviderConfig { name: "Ollama (local)", base_url, api_key: None, model: DEFAULT_MODEL })
}
