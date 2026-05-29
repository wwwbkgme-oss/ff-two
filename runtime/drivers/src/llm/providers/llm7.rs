use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://api.llm7.io/v1";
pub const DEFAULT:  &str = "default";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("LLM7_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "LLM7", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
