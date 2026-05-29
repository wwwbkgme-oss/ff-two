# Contributing to ForgeFabrik DevStudio

## Voraussetzungen

- Rust stable (aktuelle Version via `rustup update`)
- Docker + Docker Compose (für `make docker-*` Targets)
- Node.js ≥ 18 + npm (für `infra/` Pulumi IaC)
- Optional: Ollama (für lokale LLM-Tests ohne API-Key)

## Setup

```bash
git clone https://github.com/wwwbkgme-oss/ff-two
cd ff-two
cp .env.example .env

# Optional: kostenlosen LLM-Key für echte Agenten-Tests setzen
# export GROQ_API_KEY=gsk_...   (kostenloser Account: console.groq.com)

cargo build        # erstes Build (lädt Abhängigkeiten)
make check         # Typen-Check
make test          # alle Tests
```

## BKG-Architekturprinzip

Dependency-Richtung — **niemals umgekehrt**:

```
foundation  ←  domain  ←  runtime  ←  plugins
```

| Layer | Darf kennen |
|---|---|
| `foundation` | nichts (nur `std` + externe Crates) |
| `domain` | `foundation` |
| `runtime` | `foundation` + `domain` |
| `plugins` | alle internen Crates |

Vor jeder Änderung prüfen: Verletzt mein Import diese Richtung?

## Entwicklungsworkflow

```bash
make check          # cargo check --workspace (schnell)
make test           # cargo test --workspace
make test-api       # nur runtime/api Integration-Tests
make lint           # cargo clippy --workspace -- -D warnings
make fmt            # cargo fmt --all
make fmt-check      # Formatierung prüfen (CI)
make plugins        # alle Plugin-cdylibs bauen
make run            # Entwicklungs-Server (lädt .env)
make watch          # cargo-watch: Neustart bei Änderungen
```

Pre-commit-Checkliste:
```bash
make fmt && make check && make lint && make test
```

## Neue Crates

### Neues Domain-Konzept

```bash
# 1. Crate anlegen
mkdir -p domain/myfeature/src
# Cargo.toml + src/lib.rs erstellen

# 2. Workspace eintragen (Cargo.toml, [workspace.members])
# 3. Internen Alias hinzufügen ([workspace.dependencies])
# 4. README.md anlegen
# 5. CHANGELOG.md unter [Unreleased] ergänzen
```

### Neues Plugin

BKG-Namensschema:

| Ebene | Konvention | Beispiel |
|---|---|---|
| Ordner | `plugins/plugin-<name>/` | `plugins/plugin-auth/` |
| Crate-Name | `<name>` (ohne `plugin-`-Präfix) | `auth` |
| Plugin-ID | `forgefabrik.<name>` | `forgefabrik.auth` |
| Crate-Type | `["cdylib"]` oder `["cdylib", "rlib"]` | |

Pflichtdateien:
- `Cargo.toml` — `[lib] crate-type = ["cdylib"]`
- `plugin.toml` — `[plugin] id = "forgefabrik.<name>"`
- `src/lib.rs` — `plugin_info()` + `plugin_id()` C-ABI-Exports
- `README.md`

### Neuer Infrastruktur-Driver

I/O-Adapter gehören in `runtime/drivers/`, nicht in `plugins/`.  
Siehe `ARCHITECTURE.md` für die vollständige Boundary-Spec.

```rust
// runtime/drivers/src/<name>/mod.rs
use async_trait::async_trait;
use agents::SomeTrait;      // Trait aus domain/agents
use errors::AppResult;

pub struct MyDriver { /* config */ }

#[async_trait]
impl SomeTrait for MyDriver {
    async fn do_something(&self) -> AppResult<...> {
        // HTTP-Call, DB-Zugriff etc.
    }
}
```

### Neuer Free-LLM-Provider

Nur Provider mit **dauerhaft kostenlosem Tier ohne Kreditkarte**:

```rust
// runtime/drivers/src/llm/providers/myprovider.rs
use crate::llm::types::ProviderConfig;

pub const BASE_URL: &str = "https://api.myprovider.com/v1";
pub const DEFAULT:  &str = "best-free-model";

pub fn config() -> Option<ProviderConfig> {
    let k = std::env::var("MYPROVIDER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "MyProvider", base_url: BASE_URL, api_key: Some(k), model: DEFAULT })
}
```

Dann in `providers/mod.rs` → `available_providers()` ergänzen.

## Commit-Konventionen

Format: `<type>(<scope>): <kurze Beschreibung>`

| Type | Verwendung |
|---|---|
| `feat` | Neue Funktion oder Feature |
| `fix` | Bugfix |
| `docs` | Nur Dokumentation |
| `refactor` | Refactoring ohne Verhaltensänderung |
| `test` | Neue oder geänderte Tests |
| `chore` | Build, Dependencies, CI, Tooling |
| `perf` | Performance-Verbesserung |

Scope-Beispiele: `plugin-llm-free`, `runtime/server`, `domain/agents`, `infra`

Beispiele:
```
feat(plugin-llm-free): Mistral free tier provider hinzufügen
fix(router): Ollama-Erreichbarkeits-Timeout auf 3s reduzieren
docs(events): korrekte Event-Varianten dokumentieren
refactor(store): MemoryStore-Initialisierung vereinfachen
test(api): fehlende Deployment-Rollback-Tests ergänzen
chore(deps): axum auf 0.8.3 aktualisieren
```

## Pull Request-Workflow

1. Feature-Branch erstellen: `git checkout -b feat/<kurze-beschreibung>`
2. Entwickeln + lokale Validierung:
   ```bash
   make fmt && make check && make lint && make test
   ```
3. `CHANGELOG.md` unter `[Unreleased]` aktualisieren
4. PR öffnen — Titel entspricht Commit-Konvention, Beschreibung erklärt **warum**
5. CI muss grün sein (check + lint + test)

## Clippy-Lints

Wir verwenden `-D warnings` — alle Warnungen sind Fehler. Häufige Muster:

```rust
// Statt:
let _ = foo().await;
// Besser:
foo().await?;       // oder explizit ignorieren mit Kommentar

// Statt:
Arc::clone(&arc)
// Besser:
Arc::clone(&arc)    // (ist bereits idiomatisch — beibehalten)

// Statt:
.unwrap()           // in Produktions-Code verboten
// Besser:
.expect("invariant: ...")  // oder ?-Operator
```

## Lizenz

MIT. Beitragende übertragen ihre Änderungen unter der MIT-Lizenz.  
Externe Abhängigkeiten müssen MIT- oder Apache-2.0-kompatibel sein.
