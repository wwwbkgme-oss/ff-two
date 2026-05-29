# `foundation/types`

Single Source of Truth für alle Domain-Datentypen.

## BKG-Regeln

- Kein HTTP, keine DB, kein Netzwerk
- Keine Businesslogik, keine State-Mutation
- Nur Typdefinitionen, IDs, Datenstrukturen

## forge-core SYNC_CONTRACT v0.1

Enthält alle kanonischen Typen gemäß §3 und §6.

---

## Module

| Modul | Inhalt |
|---|---|
| `agent` | `Agent`, `AgentRole` (6 Rollen), `AgentStatus` |
| `canonical` | `AgentKind`, `FreeProvider` (9 Provider) |
| `deployment` | `Deployment`, `DeploymentStatus`, `DeploymentEnv`, Request-Typen |
| `forge_plugin` | `FfPluginCtx`, `FF_PLUGIN_OK/ERROR`, `abi::*`, `export_forgefabrik_plugin!` |
| `project` | `Project`, `ProjectStatus`, `ProjectTemplate`, Request-Typen |
| `sandbox` | `Sandbox`, `SandboxStatus`, `SandboxExecRequest`, `SandboxExecResult`, `CodeChange` |
| `task` | `Task`, `TaskStatus`, `TaskPriority`, `TaskClaim`, Request-Typen |
| `tick` | `WorldTick`, `TickContext`, `DeterministicRng`, `RealmId` |
| `world` | `VoxelBlock`, `VoxelKind`, `Chunk`, `BiomeType`, `WorldSnapshot` |

---

## Kanonische Typen (SYNC_CONTRACT §3)

### `WorldTick` + `TickContext`

```rust
use types::{WorldTick, TickContext, DeterministicRng, RealmId};

// Deterministischer Kontext für Domain-Funktionen
let realm = RealmId::new_v4();
let ctx   = TickContext::new(42, realm, 1);
// ctx.tick = 42, ctx.rng_seed = blake3(tick_bytes ++ realm_bytes)[..8]

// Seeded RNG statt thread_rng()
let mut rng = DeterministicRng::from_context(&ctx);
let val     = rng.next_range(100);  // [0, 100)
```

**Regel:** Domain-Code MUSS `TickContext` akzeptieren und `DeterministicRng` nutzen — **niemals** `Utc::now()` oder `thread_rng()` (SYNC_CONTRACT §4).

### `FreeProvider` + `AgentKind`

```rust
use types::{AgentKind, FreeProvider};

// RICHTIG: Provider-Gruppierung verhindert AgentKind-Explosion
let kind = AgentKind::Free(FreeProvider::Groq);
let kind = AgentKind::Free(FreeProvider::Ollama);
let kind = AgentKind::Anthropic;

// FALSCH: nie ein eigenes top-level Variant für Provider
// AgentKind::Groq  ← VERBOTEN (SYNC_CONTRACT §7)
```

`FreeProvider::to_string()` muss mit `AgentDriver::name()` in `runtime/drivers` übereinstimmen.

### Plugin-ABI (SYNC_CONTRACT §6)

```rust
use types::export_forgefabrik_plugin;

// In einem cdylib-Plugin:
struct MyPlugin;
impl MyPlugin {
    fn new() -> Self { Self }
    fn init(&mut self, _ctx: &types::forge_plugin::FfPluginCtx) -> Result<(), String> { Ok(()) }
    fn tick(&mut self, _tick: u64) -> Result<(), String> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), String> { Ok(()) }
}

export_forgefabrik_plugin!(MyPlugin, MyPlugin::new());
// Exportiert: ff_plugin_init, ff_plugin_tick, ff_plugin_shutdown
```

Kanonische Symbol-Namen: `abi::INIT = "ff_plugin_init"`, `abi::TICK = "ff_plugin_tick"`, `abi::SHUTDOWN = "ff_plugin_shutdown"`.

---

## Verwendung

```rust
// Standard-Typen
use types::{Project, Task, AgentRole, VoxelBlock};

// Kanonische Typen
use types::{AgentKind, FreeProvider, WorldTick, TickContext, DeterministicRng};

// Plugin-ABI
use types::{FfPluginCtx, plugin_abi, export_forgefabrik_plugin};
```

Alle Typen implementieren `Debug`, `Clone`, `Serialize`, `Deserialize`.

---

## Tests (SYNC_CONTRACT §8)

```bash
cargo test -p types
```

Enthält:
- `sync_contract_tests::deterministic_rng_same_seed_same_sequence` — §8.2
- `sync_contract_tests::tick_context_is_deterministic` — §8.2
- `sync_contract_tests::different_ticks_produce_different_seeds` — §8.2
- `sync_contract_tests::world_snapshot_roundtrip` — §8.3
- `sync_contract_tests::free_provider_display_matches_name` — §8.2
