# DEPRECATED — plugins/plugin-llm-free

Dieses Plugin wurde durch die saubere Architektur-Trennung ersetzt.

## Migration

| Alt | Neu |
|---|---|
| `plugins/plugin-llm-free/src/providers/` | `runtime/drivers/src/llm/providers/` |
| `plugins/plugin-llm-free/src/router.rs` | `runtime/drivers/src/llm/router.rs` |
| `plugins/plugin-llm-free/src/client.rs` | `runtime/drivers/src/llm/client.rs` |
| `plugins/plugin-llm-free/src/agent.rs` | `domain/agents/src/roles/free_llm.rs` |
| `llm_free::agent::FreeLlmAgent` | `agents::roles::FreeLlmAgent` |
| `llm_free::agent::all_roles()` | `FreeLlmAgent::all_roles(driver)` |

## Warum

LLM-Provider-Wiring ist **Infrastruktur** (HTTP-Calls, API-Keys), kein Domain-Behavior.  
Plugins dürfen nur Domain-Behavior erweitern — nie I/O machen.

Siehe `ARCHITECTURE.md` für die vollständige Boundary-Spec.

## Verbleibt im Repo

Der Code bleibt als Referenz erhalten, ist aber aus dem Workspace entfernt
und wird nicht mehr gebaut. Kann nach einem Übergangszeitraum gelöscht werden.
