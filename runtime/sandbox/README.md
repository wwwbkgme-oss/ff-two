# `runtime/sandbox`

`SandboxManager`-Trait und `LocalSandboxManager` — isolierte Code-Ausführungsumgebungen.

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

## SandboxExecRequest / SandboxExecResult

```rust
pub struct SandboxExecRequest {
    pub command:      String,
    pub args:         Vec<String>,
    pub env:          HashMap<String, String>,
    pub timeout_secs: Option<u64>,
}

pub struct SandboxExecResult {
    pub exit_code:   i32,
    pub stdout:      String,
    pub stderr:      String,
    pub duration_ms: u64,
    pub changes:     Vec<CodeChange>,
}
```

## LocalSandboxManager

Prozessbasierter Manager — nutzt temporäre Verzeichnisse + `tokio::process::Command`.

```rust
use sandbox::LocalSandboxManager;

let manager: Arc<dyn SandboxManager> = Arc::new(LocalSandboxManager::new(settings.sandbox.clone()));
```

### Konfiguration

| ENV | Default | Beschreibung |
|---|---|---|
| `DEVSTUDIO_SANDBOX_MAX` | `10` | Max. parallele Sandboxes |
| `DEVSTUDIO_SANDBOX_TIMEOUT_SECS` | `300` | Execution-Timeout (s) |
| `DEVSTUDIO_SANDBOX_WORK_DIR` | `/tmp/devstudio/sandboxes` | Basisverzeichnis |
| `DEVSTUDIO_SANDBOX_USE_DOCKER` | `false` | Docker-Isolation aktivieren |

### Verzeichnis-Layout

```
/tmp/devstudio/sandboxes/
└── {sandbox-uuid}/     ← Arbeitsverzeichnis eines Sandbox-Prozesses
```

### Snapshots (Local-Modus)

`snapshot()` gibt einen SHA-256-Hash aus Pfad + Timestamp zurück.  
`restore()` ist ein No-Op — kein Git-Tracking im lokalen Modus.

## Geplante Backends

- `DockerSandboxManager` — vollständige Container-Isolation via `bollard` (rustls, kein System-OpenSSL)

Siehe `NEXT.md` für Priorisierung.
