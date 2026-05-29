# `runtime/server`

Binärer Einstiegspunkt für ForgeFabrik DevStudio (`devstudio`).

## Aufgabe

Verdrahtet alle Crates zu einem laufenden HTTP-Server:

1. Konfiguration aus `DEVSTUDIO_*` ENV-Variablen / `.env`
2. Agenten-Modus bestimmen (`has_free_providers()`)
3. `AgentRegistry` mit 6 Rollen-Plugins (Free-LLM oder Anthropic)
4. `Orchestrator` mit konfiguriertem Konsens-Threshold
5. `MemoryStore` + `MemoryQueue` + `LocalSandboxManager` (Dev-Standard)
6. `AppState` zusammenbauen
7. Axum-Server via `runtime/api` starten

## Agenten-Modi

| Bedingung | Modus | Agenten |
|---|---|---|
| `GROQ_API_KEY` / `OPENROUTER_API_KEY` / … gesetzt | **Free-LLM** | `FreeLlmAgent` (alle 6 Rollen) |
| `DEVSTUDIO_USE_FREE_LLM=true` | **Free-LLM** | `FreeLlmAgent` + Ollama-Fallback |
| Nur `ANTHROPIC_API_KEY` gesetzt | **Standard** | `CodingAgent` (Claude) + einfache Agents |
| kein Key | **Mock** | Alle Agents im Mock-Modus |

## Starten

```bash
# Mit kostenlosen Agenten (Groq Free Tier)
GROQ_API_KEY=gsk_... cargo run --bin devstudio

# Mit Ollama lokal
DEVSTUDIO_USE_FREE_LLM=true cargo run --bin devstudio

# Mit Anthropic Claude
ANTHROPIC_API_KEY=sk-ant-... cargo run --bin devstudio

# Release-Build
cargo build --release --bin devstudio
./target/release/devstudio

# Via Make
make run
make build
```

## `build_app_state`

```rust
pub fn build_app_state(s: Settings, use_free_llm: bool) -> Result<AppState>
```

Öffentlich — kann von Tests oder anderen Binaries genutzt werden.

- `use_free_llm = true` → alle 6 Rollen nutzen `FreeLlmAgent` (forgefabrik.llm-free)
- `use_free_llm = false` → `CodingAgent` (Anthropic), Rest: einfache Agents

## `has_free_providers`

```rust
fn has_free_providers() -> bool
```

Gibt `true` zurück wenn `DEVSTUDIO_USE_FREE_LLM=true` oder mindestens einer der Keys
`OPENROUTER_API_KEY`, `GROQ_API_KEY`, `CEREBRAS_API_KEY`, `SAMBANOVA_API_KEY`, `LLM7_API_KEY`
gesetzt ist.

## Produktions-Upgrade

Für Produktion:
- `MemoryStore` → `PostgresStore`
- `MemoryQueue` → `RedisQueue`
- `LocalSandboxManager` → `DockerSandboxManager`

Keine Änderung an `runtime/api` oder Domain-Crates nötig.
