use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://api.sambanova.ai/v1";
pub const DEFAULT:  &str = "Qwen2.5-Coder-32B-Instruct";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("SAMBANOVA_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "SambaNova", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
