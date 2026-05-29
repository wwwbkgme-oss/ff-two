# ForgeFabrik DevStudio

> **Multi-Agent Software Creation in a Shared Creative Space**

Ein autonomes Entwicklungs-Framework, in dem KI-Agenten in einer Voxel-Welt kollaborieren, um Software-Projekte zu bauen, zu testen und zu deployen.

## Überblick

DevStudio ist ein Rust-Workspace nach dem BKG-Architekturprinzip:

```
foundation  →  Typen, Events, Fehlerverträge  (kein Infra, keine Logik)
domain      →  Fachdomänen, one concept = one crate
runtime     →  Infrastruktur, I/O, Außenwelt
plugins     →  Runtime-ladbare cdylib-Erweiterungen
infra       →  Pulumi IaC (AWS ECS Fargate)
```

**Dependency-Richtung:** `foundation ← domain ← runtime` — nie umgekehrt.

## Workspace-Struktur

| Layer | Crate | Aufgabe |
|---|---|---|
| foundation | `types` | Alle Domain-Datentypen, IDs, Request/Response-Structs |
| foundation | `events` | Event-Definitionen — Single Source of Truth |
| foundation | `errors` | `AppError`, `AppResult` — kein HTTP, keine Infra |
| domain | `world` | Voxel-Weltzustand, Chunks, SSE-Broadcast |
| domain | `agents` | `DevRolePlugin`-Trait, 6 Rollen, `Orchestrator` |
| domain | `security` | Statische Code-Analyse, Regeln, Findings |
| domain | `deployment` | Deployment-Pipeline-Manager, Konsenslogik |
| runtime | `config` | Settings-Loader (`DEVSTUDIO_*` ENV) |
| runtime | `store` | `Store`-Trait + `MemoryStore` |
| runtime | `queue` | `TaskQueue`-Trait + `MemoryQueue` |
| runtime | `sandbox` | `SandboxManager`-Trait + `LocalSandboxManager` |
| runtime | `api` | Axum HTTP-Router, Handler, `AppState` |
| runtime | `server` | Binärer Einstiegspunkt (`devstudio`) |
| plugins | `world-runtime` | `forgefabrik.world` cdylib |
| plugins | `agents-runtime` | `forgefabrik.agents` cdylib |
| plugins | `gm` | `forgefabrik.gm` cdylib |
| plugins | `economy` | `forgefabrik.economy` cdylib |

## Schnellstart

```bash
# 1. Konfiguration
cp .env.example .env
# Optional: ANTHROPIC_API_KEY setzen für echte KI-Agenten

# 2. Entwicklungs-Server starten
make run

# 3. Oder via Docker
make docker-up
```

Server läuft auf `http://localhost:8080`.

## API-Referenz (Kurzform)

```
GET  /health                              Healthcheck
GET  /ready                               Readiness-Check

POST /projects                            Projekt anlegen
GET  /projects                            Alle Projekte
GET  /projects/{id}                       Projekt abrufen
PUT  /projects/{id}                       Projekt aktualisieren
DEL  /projects/{id}                       Projekt archivieren

POST /projects/{id}/tasks                 Task anlegen + in Queue einreihen
GET  /projects/{id}/tasks                 Tasks eines Projekts
GET  /tasks/{id}                          Task abrufen
PUT  /tasks/{id}                          Task aktualisieren
POST /tasks/{id}/assign                   Task einem Agenten zuweisen
POST /tasks/{id}/consensus                Konsens-Vote abgeben

GET  /projects/{id}/world                 Voxel-Weltzustand
GET  /projects/{id}/world/stream          SSE-Stream (Echtzeit-Events)
GET  /projects/{id}/chunks/{cx}/{cz}      Einzelner Chunk
GET  /projects/{id}/structures            Alle Strukturen
POST /projects/{id}/snapshots             Snapshot erstellen
GET  /projects/{id}/snapshots             Snapshots auflisten

POST /sandbox/dev                         Sandbox erstellen
GET  /sandbox/{id}                        Sandbox abrufen
DEL  /sandbox/{id}                        Sandbox zerstören
POST /sandbox/{id}/execute                Befehl ausführen
GET  /sandbox/{id}/diff                   Änderungs-Diff
POST /sandbox/{id}/snapshot               Snapshot erstellen

POST /security/scan                       Code-Scan
GET  /projects/{id}/reviews               Reviews abrufen

POST /projects/{id}/deploy                Deployment starten
GET  /projects/{id}/deployments           Deployments auflisten
GET  /deployments/{id}                    Deployment abrufen
POST /deployments/{id}/rollback           Rollback
```

## Konfiguration

Alle Settings über `DEVSTUDIO_*` Umgebungsvariablen. Vollständige Referenz in `.env.example`.

```bash
DEVSTUDIO_SERVER_PORT=8080
DEVSTUDIO_AGENT_MODEL=claude-opus-4-5
ANTHROPIC_API_KEY=sk-ant-...
```

Ohne `ANTHROPIC_API_KEY` laufen Agenten im deterministischen Mock-Modus.

## Make-Targets

```bash
make build        # Release-Build
make test         # Alle Tests (Unit + Integration)
make check        # Typen-Check
make lint         # Clippy
make fmt          # Formatierung
make docker-build # Docker-Image
make docker-up    # Server via docker-compose
make run          # Entwicklungs-Server
```

## Plugins

Runtime-ladbare cdylib-Erweiterungen in `plugins/`:

| Ordner | Crate | Plugin-ID | Funktion |
|---|---|---|---|
| `plugins/plugin-world` | `world-runtime` | `forgefabrik.world` | Voxel-Koordinaten, C-ABI |
| `plugins/plugin-agents` | `agents-runtime` | `forgefabrik.agents` | Aufgabenvergabe, Konsens |
| `plugins/plugin-gm` | `gm` | `forgefabrik.gm` | Regelwerk, GM-Events |
| `plugins/plugin-economy` | `economy` | `forgefabrik.economy` | Token-Budget, Rate-Limiting |

Jedes Plugin exportiert `plugin_info()` und `plugin_id()` über ein stabiles C-ABI (`#[repr(C)]`).

## Infrastructure

Pulumi TypeScript IaC in `infra/` für AWS ECS Fargate.

```bash
cd infra
pulumi stack init dev
pulumi up
```

Provisioniert: ECR, VPC, ECS Cluster, IAM, ALB, ECS Service.

## Lizenz

MIT
