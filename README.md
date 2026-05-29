# ForgeFabrik DevStudio

> **Multi-Agent Software Creation in a Shared Creative Space**

An autonomous development environment where coding agents collaborate in a voxel world to build, test, and deploy software projects.

## Architecture

This is a Cargo workspace with one-concept-per-crate:

| Crate | Responsibility |
|---|---|
| `devstudio-types` | All shared domain models |
| `devstudio-errors` | `AppError`, `AppResult` |
| `devstudio-config` | Settings + configuration loading |
| `devstudio-store` | `Store` trait + `MemoryStore` / `PostgresStore` |
| `devstudio-queue` | `TaskQueue` trait + `MemoryQueue` / `RedisQueue` |
| `devstudio-world` | Voxel world state engine + SSE broadcast |
| `devstudio-sandbox` | `SandboxManager` trait + `LocalSandboxManager` / `DockerSandboxManager` |
| `devstudio-security` | Static-analysis scanner + finding rules |
| `devstudio-deployment` | Deployment pipeline manager |
| `devstudio-agents` | `DevRolePlugin` trait + all 6 agent roles + `Orchestrator` |
| `devstudio-api` | Axum HTTP router, handlers, middleware, `AppState` |

## Quick start

```bash
cp .env.example .env
docker compose up -d
cargo run
```

## API

Full OpenAPI spec at `/api-docs` when running. Key endpoints:

```
POST /projects            Create a new project world
POST /projects/{id}/tasks Spawn a development task
GET  /projects/{id}/world Voxel world state
GET  /projects/{id}/world/stream  SSE real-time events
POST /sandbox/dev         Create an isolated sandbox
POST /security/scan       Scan code changes
POST /projects/{id}/deploy Deploy to staging/production
```

## Configuration

All settings via `DEVSTUDIO__*` environment variables (double-underscore separator):

```
DEVSTUDIO__SERVER__PORT=8080
DEVSTUDIO__AGENTS__ANTHROPIC_API_KEY=sk-ant-...
DEVSTUDIO__DATABASE__URL=postgres://...
```

## License

MIT
