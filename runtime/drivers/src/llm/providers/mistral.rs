use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://api.mistral.ai/v1";
pub const DEFAULT:  &str = "mistral-small-latest";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("MISTRAL_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "Mistral", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
