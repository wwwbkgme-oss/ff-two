//! Kanonische Enum-Typen — forge-core SYNC_CONTRACT v0.1 §3 + §7.
//!
//! `FreeProvider` und `AgentKind` sind IDENTISCH mit forge-core.
//! Umbenennung = Breaking Change → Contract-Version bumpen.

use serde::{Deserialize, Serialize};

/// Kanonischer Typ: freie LLM-Provider-Varianten.
///
/// SYNC_CONTRACT §7: Neue Provider immer hier ergänzen,
/// niemals als neues `AgentKind`-Variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeProvider {
    OpenRouter,
    Groq,
    Cerebras,
    SambaNova,
    Mistral,
    Gemini,
    Llm7,
    Ollama,
    NvidiaNim,
}

impl std::fmt::Display for FreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OpenRouter => "openrouter",
            Self::Groq       => "groq",
            Self::Cerebras   => "cerebras",
            Self::SambaNova  => "sambanova",
            Self::Mistral    => "mistral",
            Self::Gemini     => "gemini",
            Self::Llm7       => "llm7",
            Self::Ollama     => "ollama",
            Self::NvidiaNim  => "nvidia-nim",
        };
        write!(f, "{s}")
    }
}

/// Kanonische Agent-Semantik-Klassifikation.
///
/// Beschreibt **was** ein Agent ist, nicht welchen HTTP-Endpunkt er nutzt.
/// Infrastruktur-Details leben in `runtime/drivers` — nie hier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "provider", rename_all = "snake_case")]
pub enum AgentKind {
    /// Anthropic Claude (ANTHROPIC_API_KEY).
    Anthropic,
    /// Kostenloses LLM-Backend — gruppiert um Provider-Explosion zu vermeiden.
    Free(FreeProvider),
    /// Menschlich bedient.
    Human,
    /// Deterministischer NPC (kein LLM).
    Npc,
}
