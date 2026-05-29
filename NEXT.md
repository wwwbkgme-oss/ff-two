# Next Steps — ForgeFabrik DevStudio

Geplante Weiterentwicklung nach dem aktuellen Stand (`neo/workspace-restructure-k7x2m`).

Priorisierung: **P0** = blockiert Production, **P1** = wichtig, **P2** = wertvoll, **P3** = nice-to-have.

---

## P0 — Blocking: Production-Readiness

### Task Dispatch Loop
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

### PostgresStore
**Was fehlt:** `MemoryStore` verliert alle Daten beim Neustart.  
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

### Rate Limiting
**Was fehlt:** API hat kein Rate Limiting — DoS-anfällig.  
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

### plugin-llm-free: Mistral Free Tier
**Was fehlt:** Mistral bietet kostenlose API-Zugriffe (free account, kein CC).  
**Ziel:** `providers/mistral.rs` — `MISTRAL_API_KEY`, Modelle: `mistral-small-latest` etc.

---

### plugin-llm-free: Google AI Studio Free Tier
**Was fehlt:** Google Gemini API bietet ein großzügiges kostenloses Tier via AI Studio.  
**Ziel:** `providers/gemini.rs` — `GOOGLE_AI_STUDIO_KEY`, OpenAI-kompatibles Endpoint.

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
| Kein Task-Dispatch-Loop | `runtime/server` | Tasks landen in Queue, werden nie verarbeitet |
| `MemoryStore` nicht persistiert | `runtime/store` | Datenverlust bei Neustart |
| Keine Auth-Middleware | `runtime/api` | Alle Endpoints öffentlich |
| `MemoryQueue` single-process | `runtime/queue` | Kein Scale-Out möglich |
| `LocalSandboxManager` keine Isolation | `runtime/sandbox` | Code läuft im Host-Prozess |
| Kein CI | root | Keine automatische Qualitätssicherung |
| Rate-Limit fehlt | `runtime/api` | DoS-anfällig |
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
