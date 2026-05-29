# `runtime/server`

Binärer Einstiegspunkt für ForgeFabrik DevStudio (`devstudio`).

## Aufgabe

Verdrahtet alle Crates zu einem laufenden System:

1. Konfiguration aus `DEVSTUDIO_*` ENV-Variablen / `.env`
2. Store auto-selektieren (Postgres oder Memory, je nach `DATABASE_URL`)
3. Agenten-Modus bestimmen (`has_free_providers()`)
4. `AgentRegistry` mit 6 Rollen-Plugins verdrahten
5. `FreeProviderDriver` erstellen (I/O-Adapter aus `runtime/drivers`)
6. `FreeLlmAgent::all_roles(driver)` — 6 Agents mit Driver konstruieren
7. `AppState` zusammenbauen
8. Task-Dispatch-Loop starten (`dispatcher::spawn`)
9. Axum-HTTP-Server starten (`api::serve`)

## Module

| Datei | Aufgabe |
|---|---|
| `src/main.rs` | Einstiegspunkt, verdrahtet alles |
| `src/dispatcher.rs` | Task-Dispatch-Loop (pollt Queue, ruft Agenten auf) |

## Agenten-Modi

| Bedingung | Modus | Agenten |
|---|---|---|
| `GROQ_API_KEY` / `OPENROUTER_API_KEY` / … | **Free-LLM** | `FreeLlmAgent` + `FreeProviderDriver` |
| `DEVSTUDIO_USE_FREE_LLM=true` | **Free-LLM** | `FreeLlmAgent` + Ollama-Fallback |
| Nur `ANTHROPIC_API_KEY` | **Standard** | `CodingAgent` (Claude) |
| kein Key | **Mock** | `NullDriver` / deterministischer Mock |

## Store-Auswahl

```bash
# Postgres (persistiert):
DEVSTUDIO_DATABASE_URL=postgres://user:pw@host:5432/devstudio

# SQLite (lokal):
DEVSTUDIO_DATABASE_URL=sqlite:./devstudio.db

# Memory (default, verliert Daten bei Restart):
# kein DATABASE_URL nötig
```

## Starten

```bash
# Free-LLM (Groq, kostenlos)
GROQ_API_KEY=gsk_... cargo run --bin devstudio

# Nur Ollama lokal
DEVSTUDIO_USE_FREE_LLM=true cargo run --bin devstudio

# Postgres + Free-LLM
DATABASE_URL=postgres://... GROQ_API_KEY=gsk_... cargo run --bin devstudio

# Anthropic Claude
ANTHROPIC_API_KEY=sk-ant-... cargo run --bin devstudio

# Make
make run
make build
```

## Öffentliche API

```rust
// Async (produktionsreif — wählt Store automatisch):
pub async fn build_app_state_async(s: Settings, use_free_llm: bool) -> Result<AppState>

// Sync (Tests/Benchmarks — immer MemoryStore):
pub fn build_app_state(s: Settings, use_free_llm: bool) -> Result<AppState>
```

## Architektur-Note

`runtime/server` ist der einzige Ort, an dem **Drivers** und **Agents** verdrahtet werden:

```
runtime/drivers::FreeProviderDriver  ──┐
                                       ▼ injiziert
domain/agents::FreeLlmAgent::all_roles(driver) ──→ AgentRegistry
```

Weder `domain/agents` noch `plugins/` importieren `runtime/drivers` direkt.

## Dispatcher

`dispatcher::spawn(state)` startet einen Tokio-Hintergrundtask:
- Pollt `MemoryQueue.pop()` alle 500ms
- Wählt Plugin via `orchestrator.select_plugin(&task)`
- Führt `plugin.execute()` aus
- Persistiert Ergebnis in Store, `ack` oder `nack`
- Exponentielles Back-off bei Fehlern (bis 30s)
