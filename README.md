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
| plugins | `llm-free` | `forgefabrik.llm-free` cdylib+rlib — Free LLM Router |

## Schnellstart

```bash
# 1. Konfiguration
cp .env.example .env

# 2a. Kostenlose KI-Agenten aktivieren (kein Credit Card nötig)
#     Einen der folgenden Keys setzen — oder Ollama lokal starten:
export GROQ_API_KEY=gsk_...             # https://console.groq.com (kostenlos)
export OPENROUTER_API_KEY=sk-or-v1-...  # https://openrouter.ai/keys (kostenlos)
export SAMBANOVA_API_KEY=...            # https://cloud.sambanova.ai (kostenlos, kein CC)
# ODER: ollama pull qwen2.5-coder:7b && export DEVSTUDIO_USE_FREE_LLM=true

# 2b. Oder: Anthropic Claude verwenden
export ANTHROPIC_API_KEY=sk-ant-...

# 3. Entwicklungs-Server starten
make run

# 4. Oder via Docker
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

Alle Settings über Umgebungsvariablen. Vollständige Referenz in `.env.example`.

**Agenten-Auswahl** — automatisch anhand gesetzter Keys:

| Keys gesetzt | Agenten-Modus |
|---|---|
| `GROQ_API_KEY` / `OPENROUTER_API_KEY` / … | `FreeLlmAgent` (forgefabrik.llm-free) |
| `DEVSTUDIO_USE_FREE_LLM=true` | `FreeLlmAgent` + Ollama-Fallback |
| `ANTHROPIC_API_KEY` | `CodingAgent` (Claude) |
| kein Key | deterministischer Mock |

```bash
# Kostenlose Provider (kein CC):
GROQ_API_KEY=gsk_...
OPENROUTER_API_KEY=sk-or-v1-...
SAMBANOVA_API_KEY=...
CEREBRAS_API_KEY=csk-...
LLM7_API_KEY=...
DEVSTUDIO_USE_FREE_LLM=true   # aktiviert auch Ollama-only-Modus

# Oder: Anthropic Claude
ANTHROPIC_API_KEY=sk-ant-...

# Server
DEVSTUDIO_SERVER_PORT=8080
```

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
| `plugins/plugin-llm-free` | `llm-free` | `forgefabrik.llm-free` | Free LLM Router (OpenRouter, Groq, Cerebras, SambaNova, LLM7, Ollama) |

Alle Plugins exportieren `plugin_info()` und `plugin_id()` über ein stabiles C-ABI (`#[repr(C)]`).
`plugin-llm-free` ist zusätzlich als `rlib` statisch linkbar und ersetzt den `CodingAgent` automatisch wenn Free-Provider-Keys gesetzt sind.

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
