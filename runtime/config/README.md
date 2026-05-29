# `runtime/config`

Settings-Loader — lädt alle Konfiguration aus `DEVSTUDIO_*` Umgebungsvariablen.

## Verwendung

```rust
use config::Settings;

let settings = Settings::from_env()?;
settings.init_tracing(); // Tracing-Subscriber initialisieren

let port = settings.server.port;      // u16
let url  = settings.database.url;     // String
let key  = settings.agents.anthropic_api_key; // Option<String>
```

## Vollständige Variablen-Referenz

| Variable | Typ | Default | Beschreibung |
|---|---|---|---|
| `DEVSTUDIO_SERVER_HOST` | String | `0.0.0.0` | Bind-Adresse |
| `DEVSTUDIO_SERVER_PORT` | u16 | `8080` | HTTP-Port |
| `DEVSTUDIO_DATABASE_URL` | String | `sqlite::memory:` | Datenbank-URL |
| `DEVSTUDIO_QUEUE_CAPACITY` | usize | `1024` | Queue-Puffergröße |
| `DEVSTUDIO_REDIS_URL` | String | — | Redis-URL (optional) |
| `DEVSTUDIO_SANDBOX_MAX` | usize | `10` | Max. parallele Sandboxes |
| `DEVSTUDIO_SANDBOX_TIMEOUT_SECS` | u64 | `300` | Execution-Timeout |
| `DEVSTUDIO_SANDBOX_WORK_DIR` | String | `/tmp/devstudio/sandboxes` | Arbeitsverzeichnis |
| `DEVSTUDIO_SANDBOX_USE_DOCKER` | bool | `false` | Docker-Isolation |
| `DEVSTUDIO_SEC_BLOCK_CRITICAL` | bool | `true` | Bei Critical blockieren |
| `DEVSTUDIO_SEC_BLOCK_HIGH` | bool | `false` | Bei High blockieren |
| `DEVSTUDIO_DEPLOY_CONSENSUS` | usize | `3` | Mindest-Votes |
| `DEVSTUDIO_DEPLOY_STAGING_URL` | String | — | Staging-URL |
| `DEVSTUDIO_DEPLOY_PRODUCTION_URL` | String | — | Produktions-URL |
| `DEVSTUDIO_AGENT_MODEL` | String | `claude-opus-4-5` | LLM-Modell |
| `DEVSTUDIO_AGENT_MAX_TOKENS` | u32 | `8192` | Max. Output-Tokens |
| `DEVSTUDIO_AGENT_TEMPERATURE` | f32 | `0.7` | Sampling-Temperatur |
| `ANTHROPIC_API_KEY` | String | — | Anthropic API Key |
| `OPENAI_API_KEY` | String | — | OpenAI API Key (optional) |
| **Free LLM Provider** | | | **forgefabrik.llm-free** |
| `DEVSTUDIO_USE_FREE_LLM` | bool | `false` | Free-LLM-Modus erzwingen (auch Ollama-only) |
| `OPENROUTER_API_KEY` | String | — | OpenRouter (kostenlos: `:free`-Modelle) |
| `GROQ_API_KEY` | String | — | Groq (kostenlos: 14.400 Req/Tag) |
| `CEREBRAS_API_KEY` | String | — | Cerebras (kostenlos: 60 Req/Min) |
| `SAMBANOVA_API_KEY` | String | — | SambaNova (forever free, kein CC) |
| `LLM7_API_KEY` | String | — | LLM7.io (kostenlos: 100 Req/Std) |
| `OLLAMA_BASE_URL` | String | `http://localhost:11434/v1` | Ollama-Endpunkt (kein Key nötig) |
| `DEVSTUDIO_JWT_SECRET` | String | `change-me-...` | JWT-Signierungsschlüssel |
| `DEVSTUDIO_JWT_EXPIRY_SECS` | u64 | `86400` | Token-Gültigkeitsdauer |
| `DEVSTUDIO_LOG_LEVEL` | String | `info` | Log-Level |
| `DEVSTUDIO_LOG_JSON` | bool | `false` | JSON-Logging |

`.env`-Dateien werden automatisch geladen (via `dotenvy`).
