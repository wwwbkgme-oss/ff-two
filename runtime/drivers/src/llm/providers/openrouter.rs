use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str  = "https://openrouter.ai/api/v1";
pub const DEFAULT:  &str  = "meta-llama/llama-3.3-70b-instruct:free";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("OPENROUTER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "OpenRouter", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
