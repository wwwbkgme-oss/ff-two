use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://api.cerebras.ai/v1";
pub const DEFAULT:  &str = "llama-3.3-70b";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("CEREBRAS_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "Cerebras", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
