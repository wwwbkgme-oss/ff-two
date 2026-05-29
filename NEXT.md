# Next Steps — ForgeFabrik DevStudio

Geplante Weiterentwicklung nach dem aktuellen Stand (`neo/workspace-restructure-k7x2m`).

Priorisierung: **P0** = blockiert Production, **P1** = wichtig, **P2** = wertvoll, **P3** = nice-to-have.

---

## P0 — Blocking: Production-Readiness

### ~~Task Dispatch Loop~~ ✅ `runtime/server/src/dispatcher.rs`
**Was fehlt:** Der `Orchestrator` ist verdrahtet, aber nichts pollt die Queue und ruft Agenten auf.  
Die Tasks landen in `MemoryQueue.push()` und bleiben dort — keine automatische Verarbeitung.

**Ziel:**
```
runtime/server
└── dispatcher.rs   — spawn_dispatcher(state): JoinHandle<()>
    loop {
        task = queue.pop().await
        plugin = orchestrator.select_plugin(&task)
        result = plugin.execute(&task, &sandbox_exec_result)
        store.update_task(...)
        queue.ack(task.id)
    }
```

**Scope:** `runtime/server/src/dispatcher.rs` + Integration in `main.rs`

---

### ~~PostgresStore~~ ✅ `runtime/store/src/postgres.rs`

### Auth-Middleware
**Was fehlt (noch):** `RequireAuth`-Extractor ist verfügbar, aber noch nicht auf Routen angewendet.

---

### PostgresStore ✅ (implementiert, wartend auf Postgres-Instanz)
**Implementiert:** `DEVSTUDIO_DATABASE_URL=postgres://...` aktiviert PostgresStore automatisch.  
**Ziel:** `runtime/store/src/postgres.rs` — `sqlx` PostgreSQL-Backend hinter demselben `Store`-Trait.  
`DEVSTUDIO_DATABASE_URL=postgres://...` aktiviert ihn automatisch.

**Migration:** `sqlx` Migrations in `runtime/store/migrations/`.

---

### Auth-Middleware
**Was fehlt:** Alle API-Endpunkte sind ohne Authentifizierung erreichbar.  
**Ziel:** JWT Bearer-Token Validation in Axum-Middleware.  
`DEVSTUDIO_JWT_SECRET` ist bereits in Settings; Middleware fehlt noch.

```rust
// runtime/api/src/middleware/auth.rs
async fn require_auth(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response { ... }
```

---

## P1 — Wichtig

### RedisQueue
**Was fehlt:** `MemoryQueue` ist single-process, verliert Daten bei Neustart.  
**Ziel:** `runtime/queue/src/redis.rs` — Redis Streams als persistente Queue.  
`DEVSTUDIO_REDIS_URL=redis://...` aktiviert automatisch.

---

### DockerSandboxManager
**Was fehlt:** `LocalSandboxManager` bietet keine Prozessisolation.  
**Ziel:** `runtime/sandbox/src/docker.rs` — Container-basierte Ausführung via `bollard`.

```rust
// DEVSTUDIO_SANDBOX_USE_DOCKER=true → DockerSandboxManager
// Jede Sandbox = ein kurzlebiger Docker-Container
// Image: forgefabrik/devstudio-sandbox:latest (Alpine + Rust + Node)
```

---

### GitHub Actions CI/CD
**Was fehlt:** Kein automatisierter CI-Pipeline.  
**Ziel:** `.github/workflows/ci.yml`

```yaml
on: [push, pull_request]
jobs:
  check:   cargo check --workspace
  test:    cargo test --workspace
  lint:    cargo clippy -- -D warnings
  fmt:     cargo fmt -- --check
  infra:   cd infra && npm install && npx tsc --noEmit
```

---

### OpenAPI / Swagger UI
**Was fehlt:** Kein maschinenlesbarer API-Vertrag.  
**Ziel:** `utoipa` + `utoipa-swagger-ui` Integration in `runtime/api`.  
Swagger UI erreichbar unter `GET /api-docs`.

---

### LLM Streaming (plugin-llm-free)
**Was fehlt:** `chat_request_json()` wartet auf vollständige Antwort (kein Streaming).  
**Ziel:** `stream_chat()` in `router.rs` — Tokio async stream über SSE-fähige OpenAI-API.  
Agents können Token-für-Token-Output an Clients weiterleiten.

---

### Token-Budget-Enforcement (plugin-llm-free + plugin-economy)
**Was fehlt:** `plugin-economy` enthält bereits die Berechnungslogik, aber nichts nutzt sie.  
**Ziel:** `FreeLlmAgent` ruft `economy::estimate_cost_milli()` vor jedem API-Call auf  
und bricht ab wenn `budget_sufficient() == 0`.

---

## P2 — Wertvoll

### Dynamic Plugin Loader
**Was fehlt:** Plugins werden aktuell statisch gelinkt (oder nur als cdylib gebaut, aber nicht geladen).  
**Ziel:** `runtime/server/src/plugin_loader.rs` — lädt `*.so`/`*.dylib` aus `~/.devstudio/plugins/`  
zur Laufzeit via `libloading`.

```rust
// Scan plugin dir → dlopen → plugin_info() → register capabilities
```

---

### WebSocket — Agenten-Status-Stream
**Was fehlt:** Clients können aktuell nur `WorldEvent`-SSE empfangen.  
**Ziel:** `WS /agents/stream` — Echtzeit-Updates zu laufenden Agenten  
(`AgentEvent::TaskStarted`, `AgentEvent::TaskDone`, etc.)

---

### ~~Rate Limiting~~ ✅ `runtime/api/src/middleware/rate_limit.rs`  
**Ziel:** `tower_governor` oder eigene `tower::Layer` für IP-basiertes Rate Limiting.

---

### Metriken / Prometheus
**Was fehlt:** Kein Observability-Endpoint.  
**Ziel:** `GET /metrics` — Prometheus-Format via `metrics` + `metrics-exporter-prometheus`.  
Metriken: Request-Count, Latenz, Queue-Tiefe, Agent-Erfolgsrate, Free-LLM-Provider-Calls.

---

### Multi-Tenant / Auth Scoping
**Was fehlt:** Alle Ressourcen sind global — kein User/Org-Scoping.  
**Ziel:** `owner_id: Uuid` auf `Project`/`Task`/`Agent`; API-Responses filtern nach JWT-Claims.

---

### ~~plugin-llm-free: Mistral Free Tier~~ ✅ `providers/mistral.rs`

### ~~plugin-llm-free: Google AI Studio Free Tier~~ ✅ `providers/gemini.rs`

---

## P3 — Nice-to-Have

### Kubernetes-Manifeste
Helm-Chart oder Kustomize-Konfiguration für Kubernetes-Deployment.  
Ergänzt die vorhandene ECS-Infrastruktur in `infra/`.

### Web UI (SvelteKit)
Minimalistische Browser-UI:
- Voxel-Welt-Visualisierung (3D, WebGL)  
- Task-Board (Kanban-ähnlich)  
- Agent-Status-Dashboard  
- Real-time über SSE/WebSocket

### Plugin Marketplace
`forgefabrik.market` Plugin-Registry:  
Community-Plugins mit `forgefabrik.community.*` Namespace.

### Replay / Event Sourcing
Alle Events in Event-Store persistieren → komplette Replay-Fähigkeit.  
Jeder State kann aus dem Event-Log rekonstruiert werden.

---

## Aktuelle Architekturlücken (technische Schulden)

| Lücke | Wo | Auswirkung |
|---|---|---|
| ~~Kein Task-Dispatch-Loop~~ | ✅ | `dispatcher.rs` — pollt Queue, ruft Agenten auf |
| ~~`MemoryStore` nicht persistiert~~ | ✅ | `PostgresStore` — aktivieren mit `DATABASE_URL=postgres://...` |
| Auth-Middleware | `runtime/api` | `RequireAuth`-Extractor verfügbar, Routen noch ungeschützt |
| `MemoryQueue` single-process | `runtime/queue` | Kein Scale-Out möglich |
| `LocalSandboxManager` keine Isolation | `runtime/sandbox` | Code läuft im Host-Prozess |
| ~~Kein CI~~ | ✅ | `.github/workflows/ci.yml` — check, lint, test, infra |
| ~~Rate-Limit fehlt~~ | ✅ | `rate_limit.rs` — Sliding-Window, 300 Req/Min default |
| LLM-Streaming fehlt | `plugin-llm-free` | Agenten blockieren bis Antwort komplett |

---

## Vorgeschlagene Reihenfolge (Sprint-Planung)

```
Sprint 1 (Produktions-Minimum):
  ✦ Task Dispatch Loop
  ✦ GitHub Actions CI/CD
  ✦ Auth-Middleware (JWT)

Sprint 2 (Persistenz):
  ✦ PostgresStore
  ✦ RedisQueue

Sprint 3 (Isolation + Observability):
  ✦ DockerSandboxManager
  ✦ Metriken / Prometheus
  ✦ Rate Limiting

Sprint 4 (LLM-Verbesserungen):
  ✦ LLM Streaming
  ✦ Token-Budget-Enforcement
  ✦ Mistral + Gemini Free Tier Provider

Sprint 5 (API-Qualität):
  ✦ OpenAPI / Swagger UI
  ✦ Multi-Tenant-Scoping
  ✦ WebSocket Agenten-Status
```
