use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
pub const DEFAULT:  &str = "gemini-2.0-flash-exp";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("GOOGLE_AI_STUDIO_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "Google AI Studio", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
