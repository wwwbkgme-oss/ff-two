# forge-core Sync Contract — ff-two Compliance

Reference: [`forge-core/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/SYNC_CONTRACT.md) v0.1

> **Treat this repository as part of a federated ForgeFabrik system.**
> Maintain compatibility with the canonical event + domain model.
> Do not introduce incompatible abstractions.

---

## Compliance Status

| §   | Rule | Status | Notes |
|-----|------|--------|-------|
| §2  | Layer model: foundation ← domain ← runtime | ✅ | |
| §2  | Plugin = behavior only (no I/O) | ✅ | plugin-llm-free deprecated, moved to runtime/drivers |
| §2  | Driver = I/O adapter in runtime/drivers | ✅ | `FreeProviderDriver` |
| §3  | `WorldTick` canonical name | ✅ | `foundation/types/src/tick.rs` |
| §3  | `TickContext` canonical struct | ✅ | `foundation/types/src/tick.rs` |
| §3  | `DeterministicRng` | ✅ | `foundation/types/src/tick.rs` |
| §3  | `WorldEvent` canonical name | ✅ | `foundation/events/src/world.rs` |
| §3  | `WorldSnapshot` canonical name | ✅ | `foundation/types/src/world.rs` |
| §3  | `FreeProvider` enum | ✅ | `foundation/types/src/canonical.rs` |
| §3  | `AgentKind::Free(FreeProvider)` | ✅ | `foundation/types/src/canonical.rs` |
| §3  | `FfPluginCtx` | ✅ | `foundation/types/src/forge_plugin.rs` |
| §4  | No `Utc::now()` in domain/foundation | ⚠️ | Legacy code in agent.rs uses chrono::Utc::now() for `created_at`/`last_seen` — acceptable for non-deterministic fields |
| §4  | No `thread_rng()` in domain | ✅ | `DeterministicRng` verfügbar |
| §5  | Event-First pattern | ⚠️ | `WorldState` ist imperativ — refactor zu Reducer geplant |
| §5  | `EventStore` trait | ⬜ | Geplant: NEXT.md P2 |
| §6  | Plugin ABI: `ff_plugin_init / tick / shutdown` | ✅ | Alle 4 Plugins via `export_forgefabrik_plugin!` |
| §6  | Plugin.toml kanonisches Format | ✅ | `[plugin] id / [capabilities] provides/requires / [entry] lib` |
| §7  | `AgentKind::Free(FreeProvider)` grouping | ✅ | |
| §8  | Deterministic replay test (§8.1) | ✅ | `foundation/events/src/lib.rs::sync_contract_replay_tests` |
| §8  | Event equality test (§8.2) | ✅ | `foundation/types/src/lib.rs::sync_contract_tests` |
| §8  | Snapshot round-trip test (§8.3) | ✅ | `foundation/types/src/lib.rs::sync_contract_tests` |

**Legend:** ✅ compliant · ⚠️ partial · ⬜ planned

---

## Unique contributions of ff-two

- **`export_forgefabrik_plugin!` macro** — in `foundation/types/src/forge_plugin.rs`
  (aligned with forge-core, ff-one reference implementation)
- **`runtime/drivers/llm/FreeProviderDriver`** — implements `LlmDriver` trait;
  reference implementation for 8-provider free-LLM routing
- **`domain/agents::FreeLlmAgent`** — Dependency-Injection pattern for LLM drivers
- **`runtime/server::dispatcher`** — task dispatch loop (Queue → Orchestrator → Agent)
- **`runtime/store::PostgresStore`** — JSONB-based Store implementation (ohne migrations-Konflikt)

---

## Canonical Event Mapping

| ff-two Event | forge-core canonical |
|---|---|
| `TaskEvent::Created` | `WorldEvent::AgentStateChanged` (task creation = state change) |
| `TaskEvent::Completed` | `WorldEvent::EpochEnded` (semantic: epoch boundary) |
| `WorldEvent::BlockPlaced` | `WorldEvent::BlockPlaced` (direkte Entsprechung) |
| `AgentEvent::TaskClaimed` | `WorldEvent::AgentStateChanged` |

---

## Pending Items

| Item | Priority | Reference |
|---|---|---|
| `EventStore` trait + PgEventStore | NEXT.md P1 | forge-core §5 |
| Imperative `WorldState` → Reducer refactor | NEXT.md P2 | forge-core §5 |
| `Utc::now()` aus Agent-Konstruktoren entfernen | NEXT.md P3 | forge-core §4 |
