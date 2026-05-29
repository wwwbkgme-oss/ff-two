# `plugins/plugin-llm-free`

**Plugin-ID:** `forgefabrik.llm-free`  
**Crate-Name:** `llm-free`  
**Typ:** `cdylib + rlib` — dynamisch ladbar und statisch linkbar

Free LLM Router — Ported + rebranded von [pi-free](https://github.com/apmantza/pi-free) (MIT).

Ermöglicht allen DevStudio-Agenten die Nutzung kostenloser LLM-Provider.
Kein Credit Card. Kein Ablauf. Echter Failover.

## Kostenlose Provider (Prioritätsreihenfolge)

| # | Provider | Modelle | Limit | Setup |
|---|---|---|---|---|
| 1 | **OpenRouter** | DeepSeek R1, Llama 3.3 70B, Qwen3 14B, ... (`:free`-Suffix) | Rate-limitiert | [openrouter.ai/keys](https://openrouter.ai/keys) |
| 2 | **Groq** | Llama 3.3 70B, DeepSeek R1 Distill, QwQ 32B | 14.400 Req/Tag | [console.groq.com](https://console.groq.com) |
| 3 | **Cerebras** | Llama 3.3 70B, DeepSeek R1 Distill | 60 Req/Min | [cloud.cerebras.ai](https://cloud.cerebras.ai) |
| 4 | **SambaNova** | Llama 3.3 70B, DeepSeek R1/V3, Qwen 2.5 Coder | 20–480 RPM, kein CC | [cloud.sambanova.ai](https://cloud.sambanova.ai) |
| 5 | **LLM7.io** | `default`, `fast` (Multi-Provider-Gateway) | 100 Req/Std | [token.llm7.io](https://token.llm7.io) |
| 6 | **Ollama** | Alle lokal installierten Modelle | Unbegrenzt | [ollama.com](https://ollama.com) |

**Nicht enthalten** (da Ablaufdatum / CC benötigt):
- DeepInfra ($5 Trial-Credit, läuft ab)
- Together AI ($1 Trial-Credit)
- NVIDIA NIM (1.000 Req/Monat, dann kostenpflichtig)

## Aktivierung

```bash
# Mindestens einen Key setzen:
export GROQ_API_KEY=gsk_...              # Schnellste Option
export OPENROUTER_API_KEY=sk-or-v1-...  # Beste Modellauswahl
export SAMBANOVA_API_KEY=...             # Größte Modelle (kein CC!)
export LLM7_API_KEY=...                 # Multi-Provider-Gateway

# ODER: Ollama lokal starten (kein Key nötig)
ollama pull qwen2.5-coder:7b
export DEVSTUDIO_USE_FREE_LLM=true

# Server startet automatisch im Free-LLM-Modus:
cargo run --bin devstudio
```

Wenn ein Key gesetzt ist, loggt der Server beim Start:
```
INFO  ForgeFabrik DevStudio starting  free_llm=true
INFO  AgentRegistry: Free-LLM-Modus (forgefabrik.llm-free)
```

## Architektur

```
plugins/plugin-llm-free/src/
├── lib.rs          — cdylib C-ABI Exports (plugin_info, chat_request_json, free_string)
├── agent.rs        — FreeLlmAgent: implements DevRolePlugin für alle 6 Rollen
├── router.rs       — Failover-Router: probiert Provider in Prioritätsreihenfolge
├── client.rs       — Generischer async OpenAI-kompatibler HTTP-Client (reqwest)
├── types.rs        — ChatRequest/Response, LlmError, ProviderConfig
└── providers/
    ├── mod.rs      — available_providers() → Vec<ProviderConfig>
    ├── openrouter.rs
    ├── groq.rs
    ├── cerebras.rs
    ├── sambanova.rs
    ├── llm7.rs
    └── ollama.rs
```

## C-ABI Exports (cdylib)

```c
const PluginInfo* plugin_info();
const char*       plugin_id();

// Chat-Anfrage (blockierend, JSON I/O)
const char* chat_request_json(const char* input_json);
// input:  {"system": "...", "user": "...", "max_tokens": 4096}
// output: {"text": "...", "provider": "Groq", "model": "llama-3.3-70b-versatile"}
// error:  {"error": "all providers failed: ..."}

const char* list_free_models_json();
// output: ["groq/llama-3.3-70b-versatile", "ollama/qwen2.5-coder:7b", ...]

void free_string(char* s);  // Gibt von dieser Library allozierten String frei
```

## Rust-API (rlib)

```rust
use llm_free::agent::{FreeLlmAgent, all_roles};
use llm_free::router::chat;

// Direkte LLM-Anfrage:
let result = chat("System-Prompt", "Frage...", 4096).await?;
println!("{}: {}", result.provider, result.text);

// FreeLlmAgent für alle Rollen registrieren:
for (role, agent) in all_roles() {
    registry.register(role, agent);
}
```

## ENV-Variablen

| Variable | Beschreibung |
|---|---|
| `DEVSTUDIO_USE_FREE_LLM` | `true` → erzwingt Free-LLM-Modus |
| `OPENROUTER_API_KEY` | OpenRouter API-Key (kostenlos) |
| `GROQ_API_KEY` | Groq API-Key (kostenlos) |
| `CEREBRAS_API_KEY` | Cerebras API-Key (kostenlos) |
| `SAMBANOVA_API_KEY` | SambaNova API-Key (kostenlos, kein CC) |
| `LLM7_API_KEY` | LLM7.io Token (kostenlos) |
| `OLLAMA_BASE_URL` | Ollama-URL (Standard: `http://localhost:11434/v1`) |

## Lizenz

MIT — Ported von [pi-free](https://github.com/apmantza/pi-free) von [@apmantza](https://github.com/apmantza).
