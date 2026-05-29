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

## Starten

```rust
use api::{serve, AppState};

// AppState aufbauen ...
api::serve(state).await?;
```

## Integration-Tests

```bash
cargo test -p api
```

Tests nutzen `axum-test` mit einem In-Process-Server (kein Netzwerk).  
Test-Helper: `build_test_app(state)` baut den Router ohne zu binden.

```rust
use api::build_test_app;
use axum_test::TestServer;

let server = TestServer::new(build_test_app(test_state()))?;
let r = server.get("/health").await;
r.assert_status_ok();
```

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

## Middleware

- `TraceLayer` — strukturiertes Request-Tracing
- `CorsLayer::permissive()` — CORS (für Produktion einschränken)
- `CompressionLayer` — gzip-Komprimierung
- `TimeoutLayer(60s)` — Request-Timeout
