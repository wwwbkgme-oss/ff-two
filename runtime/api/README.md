# `runtime/api`

Axum HTTP-Router, Handler, Middleware und `AppState`.

## BKG-Regeln

- Runtime-Schicht: spricht mit der Außenwelt (HTTP)
- `IntoResponse`-Impl für `AppError` lebt hier (Orphan-Regel)
- Keine Fachlogik — delegiert an Domain-Crates

## AppState

```rust
#[derive(Clone)]
pub struct AppState {
    pub store:        Arc<dyn Store>,
    pub queue:        Arc<dyn TaskQueue>,
    pub world:        Arc<WorldState>,
    pub sandbox:      Arc<dyn SandboxManager>,
    pub scanner:      Arc<Scanner>,
    pub deployer:     Arc<DeploymentManager>,
    pub orchestrator: Arc<Orchestrator>,
    pub settings:     Arc<Settings>,
}
```

## Handler-Module

```
src/handlers/
├── projects.rs   — create, list, get_by_id, update, archive
├── tasks.rs      — create, list, get_by_id, update, assign, vote
├── world.rs      — get_state, stream_events (SSE), get_chunk, list_structures, snapshots
├── sandbox.rs    — create, get_by_id, destroy, execute, diff, snapshot
├── security.rs   — scan, list_reviews
└── deployments.rs— create, list, get_by_id, rollback
```

## Starten

```rust
use api::{serve, AppState};

api::serve(state).await?;
```

## Integration-Tests

**14 Tests** in `tests/integration.rs` via `axum-test` — kein Netzwerk, rein in-process.

```bash
cargo test -p api
```

Abgedeckte Endpoints:

| Test | Endpoint |
|---|---|
| `health_returns_ok` | `GET /health` |
| `ready_returns_ok` | `GET /ready` |
| `create_project_returns_201` | `POST /projects` |
| `create_project_empty_name_returns_400` | `POST /projects` (Validierung) |
| `list_projects_initially_empty` | `GET /projects` |
| `get_project_not_found` | `GET /projects/{id}` (404) |
| `create_and_retrieve_project` | `POST` + `GET /projects/{id}` |
| `list_projects_after_create` | `GET /projects` (count) |
| `create_task_returns_201` | `POST /projects/{id}/tasks` |
| `create_task_empty_title_returns_400` | Validierung |
| `list_tasks_initially_empty` | `GET /projects/{id}/tasks` |
| `world_state_returns_empty_initially` | `GET /projects/{id}/world` |
| `create_sandbox_returns_201` | `POST /sandbox/dev` |
| `create_sandbox_missing_project_id_returns_400` | Validierung |

## Fehlerformat

Alle Fehler folgen diesem Schema:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "project 550e8400-... not found"
  }
}
```

Vollständige Code-Liste: `foundation/errors/README.md`.

## Neue Endpoints

| Endpoint | Beschreibung |
|---|---|
| `POST /auth/token` | JWT-Token ausstellen (Dev, kein CC) |
| `GET /metrics` | Prometheus-Scrape-Endpoint (public) |

## Auth

Alle Endpoints außer `/auth/token`, `/health`, `/ready`, `/metrics` sind durch JWT gesichert:

```bash
# Token holen
TOKEN=$(curl -s -X POST http://localhost:8080/auth/token \
  -H 'Content-Type: application/json' \
  -d '{"sub":"dev","secret":"change-me-in-production-use-a-strong-random-value"}' \
  | jq -r .token)

# Geschützten Endpoint aufrufen
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/projects
```

## Middleware (Reihenfolge)

1. `TraceLayer` — strukturiertes Request-Tracing (tracing-subscriber)
2. `CorsLayer::permissive()` — CORS-Header (für Produktion einschränken)
3. `CompressionLayer` — gzip-Komprimierung
4. `TimeoutLayer(60s)` — Request-Timeout

## SSE-Stream

`GET /projects/{id}/world/stream` liefert `text/event-stream`:

```
data: {"type":"block-placed","block":{...}}
data: {"type":"file-visualized","path":"src/main.rs","block_count":1}
```

Clients können alle `WorldEvent`-Varianten empfangen. Verbindung bleibt offen bis Client trennt.
