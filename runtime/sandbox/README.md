# `runtime/sandbox`

`SandboxManager`-Trait + `LocalSandboxManager` + `DockerSandboxManager`.

## Backend-Auswahl

| Backend | Aktivierung | Eigenschaften |
|---|---|---|
| `LocalSandboxManager` | Standard | Prozess im Host, kein Docker nötig |
| `DockerSandboxManager` | `DEVSTUDIO_SANDBOX_USE_DOCKER=true` | Container-Isolation, CPU/RAM-Limits |

## SandboxManager-Trait

```rust
#[async_trait]
pub trait SandboxManager: Send + Sync + 'static {
    async fn create(&self, project_id: Uuid)                   -> AppResult<Sandbox>;
    async fn execute(&self, id: Uuid, req: SandboxExecRequest) -> AppResult<SandboxExecResult>;
    async fn snapshot(&self, id: Uuid)                          -> AppResult<String>;
    async fn restore(&self, id: Uuid, snapshot_id: &str)        -> AppResult<()>;
    async fn destroy(&self, id: Uuid)                           -> AppResult<()>;
    async fn diff(&self, id: Uuid)                              -> AppResult<String>;
}
```

## LocalSandboxManager

Prozessbasierter Manager — nutzt temporäre Verzeichnisse + `tokio::process::Command`.

**Konfiguration:**

| ENV | Default | Beschreibung |
|---|---|---|
| `DEVSTUDIO_SANDBOX_MAX` | `10` | Max. parallele Sandboxes |
| `DEVSTUDIO_SANDBOX_TIMEOUT_SECS` | `300` | Execution-Timeout (s) |
| `DEVSTUDIO_SANDBOX_WORK_DIR` | `/tmp/devstudio/sandboxes` | Basisverzeichnis |

## DockerSandboxManager

Container-basierter Manager — jede Sandbox = kurzlebiger Docker-Container.

**Sicherheit:**
- Network disabled (`NetworkDisabled = true`)
- CPU: 0.5 vCPUs (quota 50000, period 100000)
- Memory: 128 MB (+ kein Swap)
- Kein Filesystem-Sharing mit Host

**Snapshot / Restore:**
- `snapshot()` = `docker commit` → Image-Tag `forgefabrik/devstudio-snapshot:<id>`
- `restore()` = neuer Container vom Snapshot-Image

**Image:**
```bash
# Standard-Image (konfigurierbar via DEVSTUDIO_SANDBOX_IMAGE):
forgefabrik/devstudio-sandbox:latest

# Minimales Image bauen (Alpine + bash + python3 + node):
docker build -t forgefabrik/devstudio-sandbox:latest -f Dockerfile.sandbox .
```

**Auto-Select:**
```bash
DEVSTUDIO_SANDBOX_USE_DOCKER=true cargo run --bin devstudio
# Startup-Log: "Sandbox: DockerSandboxManager"
```
