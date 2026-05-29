# AGENTS.md — ff-two (ForgeFabrik DevStudio)

**Treat this repository as part of a federated ForgeFabrik system.**

Maintain compatibility with the canonical event + domain model defined in
[forge-core](https://github.com/wwwbkgme-oss/forge-core).
Do not introduce incompatible abstractions.

---

## This repo's role

`ff-two` is the **DevStudio / Multi-agent software development** node of the
ForgeFabrik federation.

| Responsibility | Location |
|---|---|
| Voxel world state (project workspace) | `domain/world` |
| AI agent roles (6 dev roles + orchestrator) | `domain/agents` |
| Static code security scanning | `domain/security` |
| Deployment pipeline + consensus logic | `domain/deployment` |
| Free-tier LLM adapter (FreeProviderDriver) | `runtime/drivers` |
| HTTP API (Axum) + AppState | `runtime/api` |
| Task queue | `runtime/queue` |
| Store trait + Postgres/Memory impl | `runtime/store` |
| Sandbox (Docker / local process) | `runtime/sandbox` |
| AWS ECS Fargate IaC (Pulumi TS) | `infra/` |

---

## Canonical types implemented here

| Canonical name | Local equivalent | File |
|---|---|---|
| `Agent` | `Agent` | `foundation/types/src/agent.rs` |
| `WorldState` | `WorldState` + `WorldSnapshot` | `foundation/types/src/world.rs` |
| `WorldEvent` | `WorldEvent` (+ `AgentEvent`, `DeploymentEvent`, …) | `foundation/events/src/world.rs` |
| `ForgeError` | `AppError` | `foundation/errors/src/lib.rs` |
| `ForgeFabrikPlugin` | `PluginInfo` | `plugins/*/src/lib.rs` |

---

## Layer rules

```
foundation → domain → runtime → plugins
```

| Layer | Allowed | Forbidden |
|---|---|---|
| `foundation/` | types, events, errors | I/O, randomness, business logic |
| `domain/` | deterministic logic, traits | HTTP, DB, `Utc::now()`, `thread_rng()` |
| `runtime/` | I/O, HTTP, DB, processes, plugins | domain business logic |
| `plugins/` | domain-behaviour extensions | I/O of any kind |
| `infra/` | Pulumi IaC (TypeScript) | application code |

---

## Plugin vs Driver boundary

```
plugins/         = domain-behaviour extensions  (pure, no I/O)
runtime/drivers/ = infrastructure I/O adapters  (HTTP, transport)
```

`plugins/plugin-llm-free` is **deprecated** and must not be re-created.
LLM provider adapters belong in `runtime/drivers/`.

---

## Plugin ABI migration

`ff-two` currently exports `plugin_info()` / `plugin_id()` symbols.
The canonical ABI (forge-core) uses `ff_plugin_info()` / `ff_plugin_init()` /
`ff_plugin_tick()` / `ff_plugin_shutdown()`.

Migration path: add `ff_plugin_*` exports alongside existing symbols, then
remove old symbols in the next breaking release.
See [`forge-core/docs/PLUGIN_ABI.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/PLUGIN_ABI.md).

---

## Event-First mandate

```
Events are truth. State is projection.
Command → Event → Reducer → State Projection
```

---

## Determinism rules

Forbidden in `domain/` and `foundation/`:

- `chrono::Utc::now()` used to affect state
- `rand::thread_rng()` — use seeded RNG only
- Global mutable state

---

## Federation links

- [`forge-core`](https://github.com/wwwbkgme-oss/forge-core) — canonical definitions
- [`docs/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/SYNC_CONTRACT.md) — federation-wide sync contract
- [`docs/PLUGIN_ABI.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/PLUGIN_ABI.md) — canonical plugin ABI
