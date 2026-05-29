# `plugins/plugin-economy`

**Plugin-ID:** `forgefabrik.economy`  
**Crate-Name:** `economy`  
**Typ:** `cdylib` — runtime-loadable Dynamic Library

Ressourcen-Ökonomie Plugin: Token-Budgets, Rate-Limiting und Kosten-Tracking für KI-Agenten.

## C-ABI Exports

```c
const PluginInfo* plugin_info();
const char*       plugin_id();

// Kosten schätzen (in Milli-Credits; 1000 = 1 Credit)
uint64_t estimate_cost_milli(uint8_t tier, uint64_t input_toks, uint64_t output_toks);

// Budget prüfen
uint8_t budget_sufficient(uint64_t budget_remaining, uint64_t estimated_cost);

// Rate-Limit Throttle-Delay (ms; 0 = kein Throttling)
uint64_t throttle_delay_ms(uint32_t calls_last_minute, uint32_t rate_limit);

// Standard-Monatsbudget
uint64_t default_monthly_budget(uint8_t agent_tier);
```

## Modell-Tiers

| Tier | Beispiel | Input (Milli-Credits / 1k Tokens) | Output |
|---|---|---|---|
| 0 (Small) | Claude Haiku | 1 | 3 |
| 1 (Medium) | Claude Sonnet | 3 | 15 |
| 2 (Large) | Claude Opus | 15 | 75 |

## Agent-Tiers (Monatsbudget)

| Tier | Budget |
|---|---|
| 0 (Dev) | 10 Credits |
| 1 (Staging) | 100 Credits |
| 2 (Production) | 500 Credits |

## Manifest

Capabilities in `plugin.toml`:
- `token_budget`, `rate_limiting`, `cost_tracking`, `quota_enforcement`, `billing_events`
