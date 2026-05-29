# `foundation/errors`

Fehlerverträge des Systems — zentrale Fehlerdefinition für alle Schichten.

## BKG-Regeln

- Kein HTTP-Status, kein Axum, kein SQLx
- `IntoResponse` lebt in `runtime/api` (Orphan-Regel)
- DB-Fehler werden von `runtime/store` nach `AppError::Internal` gemappt

## API

```rust
use errors::{AppError, AppResult};

// Varianten
AppError::NotFound(String)
AppError::Conflict(String)
AppError::BadRequest(String)
AppError::Unauthorized
AppError::Forbidden
AppError::Sandbox(String)
AppError::Agent(String)
AppError::SecurityScan(String)
AppError::Deployment(String)
AppError::Internal(String)

// Hilfsmethoden
AppError::not_found("project", id)
AppError::internal(err)

// Kurzalias
type AppResult<T> = Result<T, AppError>;
```

## HTTP-Mapping

Das HTTP-Status-Mapping ist in `runtime/api/src/errors.rs` definiert:

| Variante | HTTP-Status | Code |
|---|---|---|
| `NotFound` | 404 | `NOT_FOUND` |
| `Conflict` | 409 | `CONFLICT` |
| `BadRequest` | 400 | `BAD_REQUEST` |
| `Unauthorized` | 401 | `UNAUTHORIZED` |
| `Forbidden` | 403 | `FORBIDDEN` |
| `Sandbox` | 500 | `SANDBOX_ERROR` |
| `Agent` | 500 | `AGENT_ERROR` |
| `SecurityScan` | 422 | `SECURITY_SCAN_FAIL` |
| `Deployment` | 500 | `DEPLOYMENT_ERROR` |
| `Internal` | 500 | `INTERNAL_ERROR` |
