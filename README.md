# ForgeFabrik DevStudio

> **Multi-Agent Software Creation in a Shared Creative Space**

Ein autonomes Entwicklungs-Framework, in dem KI-Agenten in einer Voxel-Welt kollaborieren, um Software-Projekte zu bauen, zu testen und zu deployen.

## Überblick

DevStudio ist ein Rust-Workspace nach dem BKG-Architekturprinzip:

```
foundation  →  Typen, Events, Fehlerverträge  (kein I/O, keine Logik)
domain      →  Fachdomänen, Traits, pure Logik  (kein I/O)
runtime     →  Infrastruktur, I/O, HTTP, DB, Prozesse
  └─ drivers/ → Infrastruktur-Adapter (LLM, künftig: weitere APIs)
plugins     →  Domain-Behavior-Erweiterungen  (kein I/O — nur Verhalten)
infra       →  Cloud-Provisionierung (Pulumi IaC, AWS ECS Fargate)
```

**Dependency-Richtung:** `foundation ← domain ← runtime` — nie umgekehrt.  
**Plugin vs. Driver:** Plugins erweitern *Verhalten*, Drivers implementieren *I/O*. Siehe `ARCHITECTURE.md`.

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
| runtime | `store` | `Store`-Trait + `MemoryStore` + `PostgresStore` |
| runtime | `queue` | `TaskQueue`-Trait + `MemoryQueue` |
| runtime | `sandbox` | `SandboxManager`-Trait + `LocalSandboxManager` |
| runtime | `api` | Axum HTTP-Router, Handler, Middleware, `AppState` |
| runtime | `drivers` | Infra-Adapter: `FreeProviderDriver` (LLM I/O) |
| runtime | `server` | Binärer Einstiegspunkt (`devstudio`), Dispatcher |
| plugins | `world-runtime` | `forgefabrik.world` — Voxel-Behavior cdylib |
| plugins | `agents-runtime` | `forgefabrik.agents` — Agenten-Behavior cdylib |
| plugins | `gm` | `forgefabrik.gm` — Game-Master-Regelwerk cdylib |
| plugins | `economy` | `forgefabrik.economy` — Token-Budget cdylib |

## forge-core Sync Contract

Dieses Repo ist Teil des **ForgeFabrik-Federations-Systems**.  
Compliance-Status: [`FORGE_CORE_SYNC.md`](FORGE_CORE_SYNC.md)  
Kanonischer Kernel: [`forge-core`](https://github.com/wwwbkgme-oss/forge-core)  
Architektur-Spec: [`ARCHITECTURE.md`](ARCHITECTURE.md)

---

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

## Neue Features (aktuell)

| Feature | Aktivierung |
|---|---|
| **PostgresStore** | `DEVSTUDIO_DATABASE_URL=postgres://...` (Auto-Select) |
| **Task-Dispatch-Loop** | Automatisch — pollt Queue, ruft Agenten auf |
| **JWT Auth** | `POST /auth/token` + `RequireAuth`-Extractor |
| **Rate Limiting** | 300 Req/Min/IP (ENV: `DEVSTUDIO_RATE_LIMIT_PER_MIN`) |
| **Free LLM** | `GROQ_API_KEY` o. ä. → `FreeLlmAgent` (8 Provider, Failover) |
| **SYNC_CONTRACT §8** | `cargo test -p types -p events` → 6 Pflicht-Tests |

---

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

Plugins erweitern **Domain-Verhalten** — sie machen **keinen I/O** (kein HTTP, keine DB).

| Ordner | Crate | Plugin-ID | Funktion |
|---|---|---|---|
| `plugins/plugin-world` | `world-runtime` | `forgefabrik.world` | Voxel-Koordinaten, Block-Typen |
| `plugins/plugin-agents` | `agents-runtime` | `forgefabrik.agents` | Aufgabenvergabe, Konsens |
| `plugins/plugin-gm` | `gm` | `forgefabrik.gm` | Regelwerk, Szenario-Events |
| `plugins/plugin-economy` | `economy` | `forgefabrik.economy` | Token-Budget, Throttling |

Alle Plugins exportieren `plugin_info()` und `plugin_id()` über stabiles C-ABI (`#[repr(C)]`).

> **`plugins/plugin-llm-free` ist deprecated** — LLM-Provider-Wiring ist Infrastruktur, kein Plugin.  
> Ersetzt durch `runtime/drivers/llm/` + `domain/agents::FreeLlmAgent`. Siehe `ARCHITECTURE.md`.

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
