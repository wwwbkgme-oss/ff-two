# `runtime/sandbox`

`SandboxManager`-Trait und `LocalSandboxManager` — isolierte Ausführungsumgebungen.

## SandboxManager-Trait

```rust
#[async_trait]
pub trait SandboxManager: Send + Sync + 'static {
    async fn create(&self, project_id: Uuid)                  -> AppResult<Sandbox>;
    async fn execute(&self, id: Uuid, req: SandboxExecRequest) -> AppResult<SandboxExecResult>;
    async fn snapshot(&self, id: Uuid)                         -> AppResult<String>;
    async fn restore(&self, id: Uuid, snapshot_id: &str)       -> AppResult<()>;
    async fn destroy(&self, id: Uuid)                          -> AppResult<()>;
    async fn diff(&self, id: Uuid)                             -> AppResult<String>;
}
```

## LocalSandboxManager

Prozessbasierter Sandbox-Manager — nutzt temporäre Verzeichnisse + `tokio::process`.

```rust
use sandbox::LocalSandboxManager;
use config::SandboxSettings;

let manager = LocalSandboxManager::new(settings.sandbox.clone());
```

### Konfiguration

| Setting | ENV | Default |
|---|---|---|
| `max_concurrent` | `DEVSTUDIO_SANDBOX_MAX` | `10` |
| `timeout_secs` | `DEVSTUDIO_SANDBOX_TIMEOUT_SECS` | `300` |
| `work_dir` | `DEVSTUDIO_SANDBOX_WORK_DIR` | `/tmp/devstudio/sandboxes` |
| `use_docker` | `DEVSTUDIO_SANDBOX_USE_DOCKER` | `false` |

### Snapshots

Im Local-Modus ist `snapshot()` ein SHA-256-Hash aus Pfad + Zeitstempel.  
`restore()` ist ein No-Op (kein Git-Tracking im lokalen Modus).

## Geplante Backends

- `DockerSandboxManager` — Container-Isolation via `bollard` (rustls, kein OpenSSL)
