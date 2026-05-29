# `plugins/plugin-gm`

**Plugin-ID:** `forgefabrik.gm`  
**Crate-Name:** `gm`  
**Typ:** `cdylib` — runtime-loadable Dynamic Library

Game-Master Plugin: steuert Spielregeln, Szenario-Events und narrative Abläufe.

## C-ABI Exports

```c
const PluginInfo* plugin_info();
const char*       plugin_id();

// Eingriffs-Score (0 = kein Eingriff, 100 = sofort)
uint8_t intervention_score(
    uint32_t blocked_agents,
    uint32_t failed_deploys,
    uint32_t security_alerts
);

// Empfohlene Aktion
// 0=Abwarten, 1=Neu zuweisen, 2=Security-Scan, 3=Pause, 4=Rollback
uint8_t recommend_action(uint8_t intervention);

// Deterministischer Szenario-Seed (für Replays/Tests)
uint64_t scenario_seed(uint64_t project_id_low, uint64_t tick);
```

## Manifest

Capabilities in `plugin.toml`:
- `rule_engine`, `scenario_scripting`, `event_injection`
- `narrative_control`, `npc_behavior`
