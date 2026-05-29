# Changelog

Alle wesentlichen Änderungen an ForgeFabrik DevStudio werden in dieser Datei dokumentiert.

Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.0.0/).

---

## [Unreleased] — branch `neo/workspace-restructure-k7x2m`

### Production-Readiness v2 (neueste Commits)
- `runtime/api/src/middleware/auth.rs` — `require_auth` Middleware (Router-Level JWT)
- `runtime/api/src/router.rs` — Auth auf alle geschützten Routen angewendet (P0 ✅)
- `runtime/api/tests/integration.rs` — 16 Tests mit JWT-Auth aktualisiert
- `domain/agents/src/budget.rs` — Token-Budget-Enforcement (P1 ✅)
- `domain/agents/src/roles/free_llm.rs` — can_afford/debit in FreeLlmAgent verdrahtet
- `runtime/api/src/metrics.rs` — Prometheus `GET /metrics` (P2 ✅)
- `domain/security/src/types.rs` — ScanRequest.project_id für Review-Speicherung
- `runtime/api/src/handlers/security.rs` — list_reviews nicht mehr Stub
- `runtime/api/src/state.rs` — AppState.scan_reviews Cache

### Production-Readiness (neueste Commits)
- `runtime/queue/src/redis.rs` — `RedisQueue`: Reliable Queue via Redis Lists, Dead-Letter, Auto-Select
- `runtime/sandbox/src/docker.rs` — `DockerSandboxManager`: Container-Isolation, CPU/RAM-Limits, Snapshot/Restore
- `runtime/server/src/main.rs` — Auto-Select: Redis-Queue + Docker-Sandbox anhand ENV-Flags

### forge-core SYNC_CONTRACT v0.1 (neueste Commits)
- `foundation/types/src/tick.rs` — `WorldTick`, `TickContext`, `DeterministicRng` (SYNC_CONTRACT §3/§4)
- `foundation/types/src/canonical.rs` — `FreeProvider` (9 Provider), `AgentKind::Free(FreeProvider)` (§7)
- `foundation/types/src/forge_plugin.rs` — `FfPluginCtx`, `export_forgefabrik_plugin!` Makro (§6)
- `foundation/types/src/lib.rs` — §8.2+§8.3 Pflicht-Tests (DeterministicRng, TickContext, Snapshot-Roundtrip)
- `foundation/events/src/lib.rs` — §8.1 Replay-Test + §8.2 Event-Equality (Single-Mutation-Path)
- Alle 4 Plugins: `ff_plugin_init/tick/shutdown` via `export_forgefabrik_plugin!` (§6)
- Alle 4 `plugin.toml`: kanonisches Format `[plugin] / [capabilities] / [entry]` (§6)
- `FORGE_CORE_SYNC.md` — Compliance-Status aller SYNC_CONTRACT-Regeln
- `ARCHITECTURE.md` — Plugin vs Driver Boundary Spec (freeze-ready)

### Added

#### Plugin: `forgefabrik.llm-free` (`plugins/plugin-llm-free`)
- Free LLM Router — ported + rebranded von [pi-free](https://github.com/apmantza/pi-free) (MIT)
- 6 kostenlose Provider ohne Credit Card, kein Ablaufdatum:
  - **OpenRouter** `:free`-Modelle (DeepSeek R1, Llama 3.3 70B, Qwen3 14B, Gemma 3 27B)
  - **Groq** Free Tier (Llama 3.3 70B, QwQ 32B, DeepSeek R1 Distill) — 14.400 Req/Tag
  - **Cerebras** Free Tier (Llama 3.3 70B, DeepSeek R1 Distill) — 60 Req/Min
  - **SambaNova** Forever-Free (Llama 3.3 70B, DeepSeek R1/V3, Qwen 2.5 Coder 32B)
  - **LLM7.io** Gateway (`default`, `fast`) — 100 Req/Std
  - **Ollama** (lokal) — unbegrenzt, kein Key, komplett privat
- `FreeLlmAgent` implementiert `DevRolePlugin` für alle 6 Rollen mit rollenspezifischen System-Prompts
- Automatischer Failover: bei Provider-Fehler → nächster Provider in Prioritätsliste
- Deterministischer Mock-Fallback wenn alle Provider nicht verfügbar
- C-ABI Exports: `plugin_info()`, `chat_request_json()`, `list_free_models_json()`, `free_string()`
- `cdylib + rlib` — dynamisch ladbar UND statisch linkbar
- `runtime/server` aktiviert Free-LLM-Modus automatisch wenn API-Key gesetzt oder `DEVSTUDIO_USE_FREE_LLM=true`

#### `runtime/server`
- Binary-Einstiegspunkt vollständig implementiert (zuvor Placeholder)
- `build_app_state(settings, use_free_llm: bool)` — Agenten-Modus wählbar
- `has_free_providers()` — prüft ENV-Keys für automatische Moduswahl

#### `plugins/` Layer (BKG-Konventionen)
- `plugins/plugin-world` → Crate `world-runtime`, ID `forgefabrik.world`
- `plugins/plugin-agents` → Crate `agents-runtime`, ID `forgefabrik.agents`
- `plugins/plugin-gm` → Crate `gm`, ID `forgefabrik.gm`
- `plugins/plugin-economy` → Crate `economy`, ID `forgefabrik.economy`
- Alle: `[lib] crate-type = ["cdylib"]`, stabiles `#[repr(C)] PluginInfo`, `plugin.toml`

#### Integration-Tests (`runtime/api/tests/integration.rs`)
- 14 Tests via `axum-test` (health, projects CRUD, tasks, world state, sandbox, deployments)

#### Docker / DevOps
- `Dockerfile` — Multi-Stage (rust:1.87 → debian:bookworm-slim)
- `docker-compose.yml` — devstudio Service mit Healthcheck + sandbox Volume
- `.env.example` — vollständige ENV-Dokumentation inkl. Free-LLM-Provider
- `Makefile` — build, test, check, fmt, lint, docker-*, run, watch, plugins

#### Infrastructure (`infra/`)
- Pulumi TypeScript IaC für AWS ECS Fargate
- Ressourcen: ECR, VPC (awsx, 2 AZs), Security Groups, ECS Cluster, IAM Roles,
  Task Definition, ALB + Target Group + Listener, ECS Service
- Stack-Konfiguration: `Pulumi.dev.yaml` (eu-central-1, 256 CPU / 512 MB)
- TypeScript compiles clean (`npx tsc --noEmit`)

#### Dokumentation
- README.md für alle 19 Crates / Plugins (vollständige Trait-Signaturen, echte Event-Varianten)
- `CHANGELOG.md` (diese Datei)
- `CONTRIBUTING.md` — Setup, BKG-Regeln, Clippy-Lints, PR-Workflow
- `NEXT.md` — priorisierter Entwicklungs-Roadmap (P0–P3, Sprint-Planung)

### Changed
- `runtime/api/Cargo.toml` — dev-dependencies erweitert für Integration-Tests
- `runtime/server/Cargo.toml` — `types` und `drivers` als Abhängigkeiten
- `Cargo.toml` (Workspace) — `runtime/drivers` als Member, `plugin-llm-free` deprecated

#### Architektur-Refactoring (Plugin vs Driver Boundary)
- `ARCHITECTURE.md` — freeze-ready Spec: Plugin = Behavior, Driver = I/O
- `domain/agents/src/llm_driver.rs` — `LlmDriver`-Trait + `NullDriver` (kein I/O)
- `domain/agents/src/roles/free_llm.rs` — `FreeLlmAgent` mit Dependency Injection
- `runtime/drivers/` — neues Crate: `FreeProviderDriver` implementiert `LlmDriver`
  - 8 Provider: OpenRouter, Groq, Cerebras, SambaNova, Mistral, Gemini, LLM7, Ollama
  - Failover-Router, OpenAI-kompatibler HTTP-Client
- `plugins/plugin-llm-free/DEPRECATED.md` — Migration zu `runtime/drivers/llm`

#### Sprint 1–4 (vorheriger Stand)
- `runtime/server/src/dispatcher.rs` — Task-Dispatch-Loop
- `runtime/store/src/postgres.rs` — PostgresStore (JSONB + Auto-Select)
- `runtime/store/migrations/001_initial.sql` — DB-Schema
- `runtime/api/src/middleware/auth.rs` — JWT `RequireAuth` + `POST /auth/token`
- `runtime/api/src/middleware/rate_limit.rs` — IP Rate Limiting (300 Req/Min)
- `.github/workflows/ci.yml` — Check, Lint, Test, Infra TypeScript
- `runtime/drivers/src/llm/providers/mistral.rs` — Mistral Free Tier
- `runtime/drivers/src/llm/providers/gemini.rs` — Google AI Studio Free Tier

---

## [0.1.0] — 2026-05-29 (commit `7537468`)

### Added
- Initialer Commit: vollständiger Rust-Workspace nach BKG-Architekturprinzip
- `foundation/types` — Domain-Datentypen (Project, Task, Agent, Voxel, Deployment, Sandbox)
- `foundation/events` — Event-Definitionen (World, Agent, Task, Sandbox, Deployment)
- `foundation/errors` — `AppError`, `AppResult`
- `domain/world` — Voxel-Weltzustand, Chunk-Management, SSE-Broadcast
- `domain/agents` — `DevRolePlugin`-Trait, 6 Rollen (Requirements/Architecture/Coding/Testing/Security/Deployment), `Orchestrator`, `AgentRegistry`
- `domain/security` — Statischer Code-Scanner, eingebaute Regeln
- `domain/deployment` — Deployment-Pipeline (Build → Test → SecurityScan → Deploy → HealthCheck)
- `runtime/config` — Settings-Loader aus `DEVSTUDIO_*` ENV-Variablen
- `runtime/store` — `Store`-Trait + `MemoryStore` (DashMap)
- `runtime/queue` — `TaskQueue`-Trait + `MemoryQueue` (mpsc + in-flight tracking)
- `runtime/sandbox` — `SandboxManager`-Trait + `LocalSandboxManager`
- `runtime/api` — Axum HTTP-Router, alle Handler, `AppState`, `IntoResponse` für `AppError`
- `CodingAgent` mit Anthropic Claude API-Integration + deterministischem Mock-Fallback
