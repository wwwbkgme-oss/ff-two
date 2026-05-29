# Next Steps — ForgeFabrik DevStudio

Roadmap nach aktuellem Stand (`neo/workspace-restructure-k7x2m`).  
Priorisierung: **P0** = blockiert Production · **P1** = wichtig · **P2** = wertvoll · **P3** = nice-to-have

---

## Erledigte Items ✅

| Sprint | Item | Datei / Scope |
|---|---|---|
| **Sync** | forge-core SYNC_CONTRACT v0.1 Compliance | `FORGE_CORE_SYNC.md` |
| **Sync** | WorldTick, TickContext, DeterministicRng | `foundation/types/src/tick.rs` |
| **Sync** | FreeProvider, AgentKind canonical grouping | `foundation/types/src/canonical.rs` |
| **Sync** | Plugin-ABI: ff_plugin_init/tick/shutdown | alle 4 plugins + `export_forgefabrik_plugin!` |
| **Sync** | SYNC_CONTRACT §8 Tests (Replay, Equality, Roundtrip) | `foundation/types` + `foundation/events` |
| 1 | Task Dispatch Loop | `runtime/server/src/dispatcher.rs` |
| 1 | GitHub Actions CI/CD | `.github/workflows/ci.yml` |
| 1 | JWT Auth-Middleware + /auth/token | `runtime/api/src/middleware/auth.rs` |
| 2 | PostgresStore (JSONB + Migrations) | `runtime/store/src/postgres.rs` |
| 2 | Auto-Select Store per DATABASE_URL | `runtime/server/src/main.rs` |
| 3 | IP Rate Limiting (Sliding-Window) | `runtime/api/src/middleware/rate_limit.rs` |
| 4 | Mistral + Google Gemini Free Provider | `plugins/plugin-llm-free/providers/` |
| **Arch** | **ARCHITECTURE.md** — Plugin vs Driver Spec | `ARCHITECTURE.md` |
| **Arch** | **LlmDriver-Trait** in domain/agents | `domain/agents/src/llm_driver.rs` |
| **Arch** | **FreeLlmAgent** mit DI in domain/agents | `domain/agents/src/roles/free_llm.rs` |
| **Arch** | **runtime/drivers** — neues Infra-Adapter-Crate | `runtime/drivers/` |
| **Arch** | **plugin-llm-free deprecated** — ersetzt durch drivers | `DEPRECATED.md` |

---

## P0 — Blocking

### Auth auf Routen anwenden
`RequireAuth`-Extractor ist implementiert (`middleware/auth.rs`), aber noch nicht auf Routen gesetzt.  
Alle `/projects`, `/tasks`, `/deployments` etc. sind noch öffentlich erreichbar.

```rust
async fn create_project(
    RequireAuth(claims): RequireAuth,   // ← hinzufügen
    State(s): State<AppState>,
    Json(b): Json<CreateProjectRequest>,
) -> ApiResult<...> { ... }
```

---

## P1 — Wichtig

### ~~RedisQueue~~ ✅ `runtime/queue/src/redis.rs`
Reliable Queue Pattern: LPUSH/RPOP + In-Flight-Hash + Dead-Letter.
Aktivierung: `DEVSTUDIO_REDIS_URL=redis://...`

```
runtime/queue/src/redis.rs   — TaskQueue-Impl via Redis Streams
DEVSTUDIO_REDIS_URL=redis://... aktiviert automatisch
```

---

### ~~DockerSandboxManager~~ ✅ `runtime/sandbox/src/docker.rs`
Jede Sandbox = kurzlebiger Docker-Container. CPU 0.5, RAM 128 MB, kein Netzwerk.
Aktivierung: `DEVSTUDIO_SANDBOX_USE_DOCKER=true`

```
runtime/sandbox/src/docker.rs  — Container via bollard (rustls)
DEVSTUDIO_SANDBOX_USE_DOCKER=true aktiviert automatisch
Image: forgefabrik/devstudio-sandbox:latest
```

---

### OpenAPI / Swagger UI
Kein maschinenlesbarer API-Vertrag.

```
utoipa + utoipa-swagger-ui
GET /api-docs  → Swagger UI
GET /api-docs/openapi.json  → OpenAPI 3.1 Spec
```

---

### LLM Streaming
`FreeLlmAgent.execute()` blockiert bis die vollständige Antwort da ist (~5–30s bei großen Modellen).

```
router.rs: stream_chat() → tokio_stream::Stream<Item = String>
Agent streamt Token-für-Token an SSE-Endpoint
GET /tasks/{id}/stream → SSE Token-Stream
```

---

### Token-Budget-Enforcement
`plugin-economy` enthält `estimate_cost_milli()` und `budget_sufficient()`, aber `FreeLlmAgent` nutzt sie nicht.

```rust
// in agent.rs vor jedem LLM-Call:
let cost = economy::estimate_cost_milli(tier, prompt_tokens, max_tokens);
if economy::budget_sufficient(remaining, cost) == 0 {
    return Err(AppError::Agent("Token-Budget erschöpft".into()));
}
```

---

## P2 — Wertvoll

### Dynamic Plugin Loader
Plugins sind aktuell statisch gelinkt. Echte Runtime-Lösung via `libloading`:

```
runtime/server/src/plugin_loader.rs
Scannt ~/.devstudio/plugins/*.so → dlopen → plugin_info() → register
```

---

### WebSocket — Agenten-Status-Stream
Clients können nur `WorldEvent`-SSE empfangen. Agenten-Status fehlt.

```
WS /agents/stream → AgentEvent::TaskStarted, TaskDone, ...
```

---

### Prometheus Metriken
Kein Observability-Endpoint.

```
metrics + metrics-exporter-prometheus
GET /metrics → Prometheus-Format
Metriken: request_count, latency_p99, queue_depth, agent_success_rate,
          llm_provider_calls, llm_provider_errors
```

---

### Multi-Tenant / Auth Scoping
Alle Ressourcen sind global — kein User/Org-Scoping.

```
owner_id: Uuid auf Project/Task/Agent
API filtert automatisch nach JWT claims.sub
```

---

## P3 — Nice-to-Have

### Kubernetes-Manifeste
Helm-Chart oder Kustomize für Kubernetes-Deployment (ergänzt ECS-Infra in `infra/`).

### Web UI (SvelteKit)
- Voxel-Welt-Visualisierung (3D, WebGL / Three.js)
- Task-Board (Kanban)
- Agent-Status-Dashboard (Real-time SSE/WS)
- Free-LLM-Provider-Status-Anzeige

### Plugin Marketplace
`forgefabrik.market` Registry — Community-Plugins unter `forgefabrik.community.*`.

### Event Sourcing / Replay
Alle Events in Event-Store persistieren → vollständige Replay-Fähigkeit.

---

## Offene Architekturlücken

| Lücke | Status | Nächster Schritt |
|---|---|---|
| Auth auf Routen | P0 | `RequireAuth` in Handler-Signaturen ergänzen |
| ~~`MemoryQueue` single-process~~ | ✅ | `RedisQueue` — `DEVSTUDIO_REDIS_URL=redis://...` |
| ~~Sandbox-Isolation~~ | ✅ | `DockerSandboxManager` — `DEVSTUDIO_SANDBOX_USE_DOCKER=true` |
| LLM blockiert | P1 | Streaming-Endpoint |
| Token-Budget ungenutzt | P1 | Economy-Plugin in FreeLlmAgent verdrahten |
| Kein OpenAPI-Spec | P1 | `utoipa` Integration |
| Keine Metriken | P2 | Prometheus-Endpoint |
| Kein Multi-Tenant | P2 | `owner_id` + JWT-Scoping |

---

## Sprint-Plan (aktualisiert)

```
Sprint 1 ✅  Task Dispatch Loop · CI/CD · Auth-Middleware
Sprint 2 ✅  PostgresStore · Store Auto-Select
Sprint 3 ✅  Rate Limiting
Sprint 4 ✅  Mistral + Gemini Free Provider

Sprint 5 (nächster):
  → Auth auf alle Routen anwenden
  → RedisQueue
  → Token-Budget-Enforcement

Sprint 6:
  → DockerSandboxManager
  → LLM Streaming
  → OpenAPI / Swagger UI

Sprint 7:
  → Prometheus Metriken
  → WebSocket Agenten-Status
  → Multi-Tenant-Scoping
```
