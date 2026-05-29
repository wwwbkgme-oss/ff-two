# `runtime/server`

Binärer Einstiegspunkt für ForgeFabrik DevStudio (`devstudio`).

## Aufgabe

Verdrahtet alle Crates zu einem laufenden HTTP-Server:

1. Konfiguration aus `DEVSTUDIO_*` ENV-Variablen / `.env`
2. `AgentRegistry` mit allen 6 Rollen-Plugins
3. `Orchestrator` mit konfiguriertem Konsens-Threshold
4. `MemoryStore` + `MemoryQueue` + `LocalSandboxManager` (Dev-Standard)
5. `AppState` zusammenbauen
6. Axum-Server via `runtime/api` starten

## Starten

```bash
# Entwicklung
cargo run --bin devstudio

# Release
cargo build --release --bin devstudio
./target/release/devstudio

# Via Make
make run
make build
```

## build_app_state

Die Funktion `build_app_state(settings: Settings) -> Result<AppState>` ist öffentlich und kann von anderen Binaries oder Test-Setups wiederverwendet werden.

## Produktions-Upgrade

Für Produktion kann `MemoryStore` durch `PostgresStore` und `MemoryQueue` durch `RedisQueue` ersetzt werden — ohne Änderung an `runtime/api` oder Domain-Crates.
