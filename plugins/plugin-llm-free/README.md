# `plugins/plugin-llm-free`

**Plugin-ID:** `forgefabrik.llm-free`  
**Crate-Name:** `llm-free`  
**Typ:** `cdylib + rlib` — dynamisch ladbar und statisch linkbar

Free LLM Router — ported + rebranded von [pi-free](https://github.com/apmantza/pi-free) (MIT).

Ermöglicht allen DevStudio-Agenten die Nutzung kostenloser LLM-Provider.  
Kein Credit Card. Kein Ablaufdatum. Echter Failover.

## Kostenlose Provider (Prioritätsreihenfolge)

| # | Provider | Modelle | Limit | Setup |
|---|---|---|---|---|
| 1 | **OpenRouter** | DeepSeek R1, Llama 3.3 70B, Qwen3 14B, Gemma 3 27B (`:free`-Suffix) | Rate-limitiert | [openrouter.ai/keys](https://openrouter.ai/keys) |
| 2 | **Groq** | Llama 3.3 70B, QwQ 32B, DeepSeek R1 Distill, Gemma 2 9B | 14.400 Req/Tag | [console.groq.com](https://console.groq.com) |
| 3 | **Cerebras** | Llama 3.3 70B, DeepSeek R1 Distill, Qwen 3 32B | 60 Req/Min | [cloud.cerebras.ai](https://cloud.cerebras.ai) |
| 4 | **SambaNova** | Llama 3.3 70B, 405B, DeepSeek R1/V3, Qwen 2.5 Coder 32B | 20–480 RPM, **kein CC** | [cloud.sambanova.ai](https://cloud.sambanova.ai) |
| 5 | **LLM7.io** | `default`, `fast` (Multi-Provider-Gateway) | 100 Req/Std | [token.llm7.io](https://token.llm7.io) |
| 6 | **Ollama** | Alle lokal installierten Modelle | Unbegrenzt | [ollama.com](https://ollama.com) |

**Bewusst ausgeschlossen** — kein dauerhafter Free-Tier:

| Provider | Grund |
|---|---|
| DeepInfra | $5 Trial-Credit, läuft nach 90 Tagen ab |
| Together AI | $1 Trial-Credit, läuft ab |
| NVIDIA NIM | 1.000 Req/Monat, danach kostenpflichtig |

## Aktivierung

```bash
# Option A: Einen Cloud-Key setzen (kostenloser Account, kein CC)
export GROQ_API_KEY=gsk_...              # Schnellste Inferenz
export OPENROUTER_API_KEY=sk-or-v1-...  # Beste Modellauswahl
export SAMBANOVA_API_KEY=...             # Größte Modelle (kein CC!)
export CEREBRAS_API_KEY=csk-...         # Schnellste großes Modell
export LLM7_API_KEY=...                 # Multi-Provider-Gateway

# Option B: Ollama lokal (vollständig privat, kein Internet nötig)
ollama pull qwen2.5-coder:7b
export DEVSTUDIO_USE_FREE_LLM=true

# Server startet automatisch im Free-LLM-Modus:
cargo run --bin devstudio
```

Server-Log bei aktivem Free-LLM-Modus:
```
INFO devstudio: ForgeFabrik DevStudio starting  free_llm=true
INFO devstudio: AgentRegistry: Free-LLM-Modus (forgefabrik.llm-free)
```

## Architektur

```
plugins/plugin-llm-free/src/
├── lib.rs             — cdylib C-ABI Exports + OnceLock Tokio-Runtime
├── agent.rs           — FreeLlmAgent: DevRolePlugin für alle 6 Rollen
│                          rollenspezifische System-Prompts (DE/EN)
│                          all_roles() → Vec<(AgentRole, Arc<FreeLlmAgent>)>
├── router.rs          — Failover-Router, Ollama-Erreichbarkeits-Check
├── client.rs          — Generischer async OpenAI-kompatibler HTTP-Client
├── types.rs           — ChatMessage/Request/Response, LlmError, ProviderConfig
└── providers/
    ├── mod.rs         — available_providers() → Vec<ProviderConfig>
    ├── openrouter.rs  — FREE_MODELS: 7 :free Modelle
    ├── groq.rs        — FREE_MODELS: 6 Modelle
    ├── cerebras.rs    — FREE_MODELS: 5 Modelle
    ├── sambanova.rs   — FREE_MODELS: 7 Modelle
    ├── llm7.rs        — FREE_MODELS: default, fast
    └── ollama.rs      — dynamische Entdeckung via /models
```

## C-ABI Exports (cdylib)

```c
const PluginInfo* plugin_info();      // statische Metadaten
const char*       plugin_id();        // "forgefabrik.llm-free\0"

// Chat-Anfrage (blockierend via statischem Tokio-Runtime)
// input:  {"system":"...","user":"...","max_tokens":4096}
// output: {"text":"...","provider":"Groq","model":"llama-3.3-70b-versatile","tokens":512}
// error:  {"error":"all providers failed: ..."}
const char* chat_request_json(const char* input_json);

// Listet alle konfigurierten freien Modelle
// output: ["groq/llama-3.3-70b-versatile","ollama/qwen2.5-coder:7b",...]
const char* list_free_models_json();

// Gibt Library-allozierten String frei — MUSS für jeden Rückgabewert aufgerufen werden
void free_string(char* s);
```

## Rust-API (rlib)

```rust
use llm_free::agent::{FreeLlmAgent, all_roles};
use llm_free::router::chat;

// Direkte LLM-Anfrage mit Failover:
let result = chat("Du bist ein Senior Engineer.", "Schreibe einen Quicksort in Rust.", 4096).await?;
println!("[{}] {}: {}", result.provider, result.model, result.text);

// Alle 6 Rollen mit FreeLlmAgent belegen:
for (role, agent) in all_roles() {
    registry.register(role, agent as Arc<dyn DevRolePlugin>);
}
```

## System-Prompts je Rolle

| Rolle | Fokus |
|---|---|
| `Requirements` | Anforderungsanalyse, User-Stories, Akzeptanzkriterien |
| `Architecture` | System-Design, Komponenten, API-Verträge |
| `Coding` | Produktionsreifer Code, Rust-Best-Practices |
| `Testing` | Tests, ≥80 % Coverage, Framework-idiomatisch |
| `Security` | OWASP Top 10, Findings, Lösungsvorschläge |
| `Deployment` | Dockerfiles, CI/CD, Kubernetes, IaC |

## Neuen Provider hinzufügen

1. `src/providers/myprovider.rs` anlegen:

```rust
use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.myprovider.com/v1";
pub const FREE_MODELS: &[FreeModel] = &[
    FreeModel { id: "my-model-7b", name: "MyModel 7B", context_window: 32_768, max_tokens: 4_096, reasoning: false },
];
pub const DEFAULT_MODEL: &str = "my-model-7b";

pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("MYPROVIDER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "MyProvider", base_url: BASE_URL, api_key: Some(key), model: DEFAULT_MODEL })
}
```

2. In `src/providers/mod.rs` registrieren:

```rust
pub mod myprovider;

pub fn available_providers() -> Vec<ProviderConfig> {
    let mut out = Vec::new();
    // ...bestehende Provider...
    if let Some(p) = myprovider::config() { out.push(p); }
    out
}
```

**Kriterium:** Nur Provider mit **dauerhaft kostenlosem Tier ohne Kreditkarte** — kein Trial-Guthaben mit Ablaufdatum.

## Troubleshooting

**„all providers failed"**  
→ Keiner der konfigurierten Provider ist erreichbar. Prüfe:
- API-Keys korrekt gesetzt? (`echo $GROQ_API_KEY`)
- Netzwerkverbindung vorhanden?
- Rate-Limit überschritten? (Groq: 100 Req/Min)

**Ollama wird nicht gefunden**  
→ `ollama serve` läuft nicht. Starte Ollama:
```bash
ollama serve &
ollama pull qwen2.5-coder:7b
```

**Mock-Fallback aktiv**  
→ Kein Key gesetzt und Ollama nicht erreichbar. Server gibt Platzhalter-Code zurück.  
→ Lösung: `export GROQ_API_KEY=...` (kostenloser Account unter console.groq.com)

**Provider antwortet mit 429 (Rate Limit)**  
→ Der Router versucht automatisch den nächsten Provider. Kein manuelles Eingreifen nötig.

## ENV-Variablen

| Variable | Default | Beschreibung |
|---|---|---|
| `DEVSTUDIO_USE_FREE_LLM` | `false` | Free-LLM-Modus erzwingen |
| `OPENROUTER_API_KEY` | — | OpenRouter (kostenlos) |
| `GROQ_API_KEY` | — | Groq (kostenlos, 14.400 Req/Tag) |
| `CEREBRAS_API_KEY` | — | Cerebras (kostenlos, 60 Req/Min) |
| `SAMBANOVA_API_KEY` | — | SambaNova (forever free, kein CC) |
| `LLM7_API_KEY` | — | LLM7.io (kostenlos, 100 Req/Std) |
| `OLLAMA_BASE_URL` | `http://localhost:11434/v1` | Ollama-Endpunkt |

## Lizenz

MIT — Ported von [pi-free](https://github.com/apmantza/pi-free) von [@apmantza](https://github.com/apmantza).
