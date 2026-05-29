# Contributing to ForgeFabrik DevStudio

## Voraussetzungen

- Rust stable (aktuelle Version via `rustup`)
- Docker (für `make docker-*` Targets)
- Node.js ≥ 18 + npm (für `infra/` Pulumi IaC)

## Setup

```bash
git clone https://github.com/wwwbkgme-oss/ff-two
cd ff-two
cp .env.example .env
# Optional: Free-LLM-Key setzen (kein CC nötig):
# export GROQ_API_KEY=gsk_...
cargo build
```

## BKG-Architekturprinzip

Jede Änderung muss die Dependency-Richtung respektieren:

```
foundation  ←  domain  ←  runtime  ←  plugins
```

- `foundation`-Crates kennen keine anderen internen Crates
- `domain`-Crates kennen nur `foundation`
- `runtime`-Crates kennen `foundation` + `domain`
- `plugins`-Crates dürfen alle internen Crates kennen

## Entwicklungsworkflow

```bash
# Typecheck (schnell)
make check

# Tests
make test

# Formatierung prüfen
make fmt-check

# Linting
make lint

# Alles auf einmal (pre-commit)
make check && make fmt && make lint && make test
```

## Neue Features

### Neues Domain-Konzept
1. Crate in `domain/<name>/` erstellen
2. In `Cargo.toml` Workspace-Member eintragen
3. Interne Crate-Alias in `[workspace.dependencies]` anlegen
4. README.md im Crate-Verzeichnis anlegen
5. CHANGELOG.md aktualisieren

### Neues Plugin
Plugins folgen dem BKG-Namensschema:

| Ebene | Konvention |
|---|---|
| Ordner | `plugins/plugin-<name>/` |
| Crate-Name | `<name>` (ohne `plugin-`-Präfix, falls kein Konflikt) |
| Plugin-ID | `forgefabrik.<name>` |
| Crate-Type | `["cdylib"]` oder `["cdylib", "rlib"]` |

Jedes Plugin benötigt:
- `Cargo.toml` mit `[lib] crate-type`
- `plugin.toml` mit `[plugin] id = "forgefabrik.<name>"`
- `src/lib.rs` mit `plugin_info()` + `plugin_id()` C-ABI-Exports
- `README.md`

### Neuer Free-LLM-Provider
In `plugins/plugin-llm-free/src/providers/` eine neue Datei anlegen:

```rust
// providers/myprovider.rs
use crate::types::{FreeModel, ProviderConfig};

pub const BASE_URL: &str = "https://api.myprovider.com/v1";
pub const FREE_MODELS: &[FreeModel] = &[/* ... */];
pub const DEFAULT_MODEL: &str = "my-model";

pub fn config() -> Option<ProviderConfig> {
    let key = std::env::var("MYPROVIDER_API_KEY").ok().filter(|s| !s.is_empty())?;
    Some(ProviderConfig { name: "MyProvider", base_url: BASE_URL, api_key: Some(key), model: DEFAULT_MODEL })
}
```

Dann in `providers/mod.rs` eintragen und `available_providers()` ergänzen.

**Wichtig:** Nur Provider mit dauerhaft kostenlosem Tier ohne Kreditkarte — kein Trial-Guthaben mit Ablaufdatum.

## Commit-Konventionen

Format: `<type>(<scope>): <description>`

| Type | Verwendung |
|---|---|
| `feat` | Neue Funktion |
| `fix` | Bugfix |
| `docs` | Nur Dokumentation |
| `refactor` | Refactoring ohne Verhaltensänderung |
| `test` | Tests |
| `chore` | Build, Dependencies, CI |

Beispiele:
```
feat(plugin-llm-free): Mistral free tier provider hinzufügen
fix(router): Ollama-Timeout von 60s auf 30s reduzieren
docs(README): Free-LLM-Quickstart ergänzen
```

## Pull Requests

1. Feature-Branch von `main` erstellen: `git checkout -b feat/mein-feature`
2. `make check && make test` muss grün sein
3. `CHANGELOG.md` unter `[Unreleased]` aktualisieren
4. PR öffnen — Beschreibung erklärt _warum_, nicht _was_
