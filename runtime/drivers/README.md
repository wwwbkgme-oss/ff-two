# `runtime/drivers`

Infrastruktur-Adapter — implementiert Traits aus `domain/` mit echtem I/O.

## BKG / ARCHITECTURE.md

```
domain/agents::LlmDriver   ←  Trait (kein I/O, pure interface)
runtime/drivers::FreeProviderDriver  →  Implementierung (HTTP, API-Keys)
```

**Niemals** von `domain/` oder `plugins/` importiert.  
Verdrahtung ausschließlich in `runtime/server/main.rs`.

## Aktuelle Driver

### `FreeProviderDriver` — LLM I/O

Implementiert `domain/agents::LlmDriver` via 8 kostenloser Provider:

| Priorität | Provider | ENV-Key | Limit |
|---|---|---|---|
| 1 | OpenRouter | `OPENROUTER_API_KEY` | :free-Modelle |
| 2 | Groq | `GROQ_API_KEY` | 14.400 Req/Tag |
| 3 | Cerebras | `CEREBRAS_API_KEY` | 60 Req/Min |
| 4 | SambaNova | `SAMBANOVA_API_KEY` | forever free, kein CC |
| 5 | Mistral | `MISTRAL_API_KEY` | freier Account |
| 6 | Google AI Studio | `GOOGLE_AI_STUDIO_KEY` | 1.500 Req/Tag |
| 7 | LLM7.io | `LLM7_API_KEY` | 100 Req/Std |
| 8 | Ollama (lokal) | kein Key | unbegrenzt |

```rust
use drivers::FreeProviderDriver;

let driver = Arc::new(FreeProviderDriver::new());
// Übergabe an FreeLlmAgent in runtime/server
```

## Verzeichnisstruktur

```
src/
├── lib.rs              — FreeProviderDriver re-exportieren
└── llm/
    ├── mod.rs          — FreeProviderDriver: impl LlmDriver
    ├── client.rs       — generischer OpenAI-kompatibler HTTP-Client
    ├── router.rs       — Failover-Router mit Ollama-Erreichbarkeits-Check
    ├── types.rs        — interne Typen (ChatRequest, ProviderConfig, …)
    └── providers/
        ├── mod.rs      — available_providers() → Vec<ProviderConfig>
        ├── openrouter.rs
        ├── groq.rs
        ├── cerebras.rs
        ├── sambanova.rs
        ├── mistral.rs
        ├── gemini.rs
        ├── llm7.rs
        └── ollama.rs
```

## Neuen Driver hinzufügen

1. Trait in `domain/` oder `foundation/` definieren (kein I/O!)
2. Modul in `runtime/drivers/src/<name>/` anlegen
3. Trait implementieren mit echtem I/O
4. In `runtime/server/main.rs` verdrahten
5. ARCHITECTURE.md → Tabelle ergänzen

## Neuen LLM-Provider hinzufügen

Datei in `src/llm/providers/myprovider.rs`:

```rust
use crate::llm::types::ProviderConfig;
pub const BASE_URL: &str = "https://api.myprovider.com/v1";
pub const DEFAULT:  &str = "best-free-model";
pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("MYPROVIDER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "MyProvider", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
```

Dann in `providers/mod.rs` → `available_providers()` ergänzen.

**Kriterium:** Nur dauerhaft kostenlos, kein CC — keine Trial-Credits mit Ablaufdatum.
