# `plugins/plugin-agents`

**Plugin-ID:** `forgefabrik.agents`  
**Crate-Name:** `agents-runtime` *(domain/agents belegt "agents")*  
**Typ:** `cdylib` — runtime-loadable Dynamic Library

## BKG-Konventionen

| Ebene | Wert |
|---|---|
| Ordner | `plugins/plugin-agents` |
| Crate-Name | `agents-runtime` |
| Plugin-ID | `forgefabrik.agents` |

## C-ABI Exports

```c
const PluginInfo* plugin_info();
const char*       plugin_id();

// Aufgabenvergabe
uint8_t can_claim(uint8_t required_role_id, uint8_t agent_role_id);

// Konsens
uint8_t consensus_met(uint32_t votes_count, uint32_t threshold);

// Priorität
uint64_t task_priority_score(uint32_t attempts, uint64_t age_secs);
```

## Rollen-IDs

| ID | Rolle |
|---|---|
| 0 | Requirements |
| 1 | Architecture |
| 2 | Coding |
| 3 | Testing |
| 4 | Security |
| 5 | Deployment |
| 255 | Beliebig (keine Einschränkung) |

## Manifest

Capabilities in `plugin.toml`:
- Alle 6 Rollen + `consensus_voting`
