//! DockerSandboxManager — Container-basierte Code-Ausführung via bollard.
//!
//! ## Aktivierung
//! `DEVSTUDIO_SANDBOX_USE_DOCKER=true` in der Umgebung setzen.
//!
//! ## Isolation
//! Jede Sandbox läuft in einem eigenen kurzlebigen Docker-Container:
//! - CPU-Limit: 0.5 CPUs
//! - Memory-Limit: 128 MB
//! - Network deaktiviert (`NetworkDisabled = true`)
//! - Nur definierte Umgebungsvariablen aus dem Request
//!
//! ## Image
//! Standard: `forgefabrik/devstudio-sandbox:latest`
//! Konfigurierbar via `DEVSTUDIO_SANDBOX_IMAGE`.
//! Minimales Image: Alpine + bash + python3 + node.
//!
//! ## Snapshot / Restore
//! `snapshot()` committet den Container-Layer zu einem Image-Tag.
//! `restore()` startet einen neuen Container vom Snapshot-Image.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CommitContainerOptions;
use bollard::Docker;
use chrono::Utc;
use futures::StreamExt;
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::{Sandbox, SandboxExecRequest, SandboxExecResult, SandboxStatus};

use super::manager::SandboxManager;

// ── Konstanten ────────────────────────────────────────────────────────────────

const DEFAULT_IMAGE:   &str = "forgefabrik/devstudio-sandbox:latest";
const CPU_QUOTA:       i64  = 50_000;   // 0.5 CPUs (100_000 = 1 CPU)
const CPU_PERIOD:      i64  = 100_000;
const MEMORY_LIMIT:    i64  = 128 * 1024 * 1024; // 128 MB

// ── DockerSandboxManager ──────────────────────────────────────────────────────

/// Container-basierter Sandbox-Manager — produktionsreife Isolation.
pub struct DockerSandboxManager {
    docker:  Docker,
    image:   String,
    timeout: Duration,
}

impl std::fmt::Debug for DockerSandboxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DockerSandboxManager")
            .field("image", &self.image)
            .finish()
    }
}

impl DockerSandboxManager {
    /// Verbindet mit dem lokalen Docker-Daemon.
    pub async fn new(timeout_secs: u64) -> AppResult<Self> {
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| AppError::Sandbox(format!("Docker-Verbindung: {e}")))?;

        let image = std::env::var("DEVSTUDIO_SANDBOX_IMAGE")
            .unwrap_or_else(|_| DEFAULT_IMAGE.to_owned());

        tracing::info!(image = %image, "DockerSandboxManager initialisiert");
        Ok(Self { docker, image, timeout: Duration::from_secs(timeout_secs) })
    }

    /// Container-Name aus Sandbox-ID ableiten.
    fn container_name(id: Uuid) -> String {
        format!("devstudio-sb-{}", &id.to_string()[..8])
    }
}

#[async_trait]
impl SandboxManager for DockerSandboxManager {
    /// Neuen Container erstellen und starten.
    async fn create(&self, project_id: Uuid) -> AppResult<Sandbox> {
        let id   = Uuid::new_v4();
        let name = Self::container_name(id);

        let options = CreateContainerOptions { name: &name, platform: None };
        let config  = Config::<String> {
            image:            Some(self.image.clone()),
            cmd:              Some(vec!["sleep".into(), "3600".into()]),  // hält Container am Leben
            network_disabled: Some(true),                                  // kein Netzwerk
            host_config:      Some(bollard::models::HostConfig {
                cpu_quota:          Some(CPU_QUOTA),
                cpu_period:         Some(CPU_PERIOD),
                memory:             Some(MEMORY_LIMIT),
                memory_swap:        Some(MEMORY_LIMIT),    // kein Swap
                oom_kill_disable:   Some(false),
                readonly_rootfs:    Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| AppError::Sandbox(format!("Container erstellen: {e}")))?;

        self.docker
            .start_container(&name, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| AppError::Sandbox(format!("Container starten: {e}")))?;

        tracing::debug!(sandbox_id = %id, container = %name, "DockerSandbox: erstellt");

        Ok(Sandbox {
            id,
            project_id,
            agent_id:        None,
            status:          SandboxStatus::Ready,
            runtime_id:      Some(name),
            work_dir:        format!("/workspace/{id}"),
            snapshot_id:     None,
            pending_changes: vec![],
            created_at:      Utc::now(),
            updated_at:      Utc::now(),
        })
    }

    /// Befehl in Container ausführen via exec.
    async fn execute(&self, sandbox_id: Uuid, req: SandboxExecRequest) -> AppResult<SandboxExecResult> {
        let name  = Self::container_name(sandbox_id);
        let start = Instant::now();

        // Umgebungsvariablen als `KEY=VALUE`-Strings
        let env: Vec<String> = req.env.iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        let exec_id = self.docker
            .create_exec(
                &name,
                CreateExecOptions {
                    cmd:          Some(std::iter::once(req.command.as_str())
                                       .chain(req.args.iter().map(String::as_str))
                                       .collect()),
                    env:          Some(env.iter().map(String::as_str).collect()),
                    working_dir:  Some("/workspace"),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| AppError::Sandbox(format!("exec create: {e}")))?
            .id;

        let timeout = req.timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.timeout);

        let result = tokio::time::timeout(timeout, async {
            let output = self.docker
                .start_exec(&exec_id, None)
                .await
                .map_err(|e| AppError::Sandbox(format!("exec start: {e}")))?;

            let mut stdout = String::new();
            let mut stderr = String::new();

            if let StartExecResults::Attached { mut output, .. } = output {
                while let Some(chunk) = output.next().await {
                    match chunk.map_err(|e| AppError::Sandbox(e.to_string()))? {
                        bollard::container::LogOutput::StdOut { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
            }
            Ok::<(String, String), AppError>((stdout, stderr))
        })
        .await
        .map_err(|_| AppError::Sandbox("execution timed out".into()))?
        .map_err(|e| AppError::Sandbox(e.to_string()))?;

        // Exit-Code aus Exec-Inspect auslesen
        let exit_code = self.docker
            .inspect_exec(&exec_id)
            .await
            .ok()
            .and_then(|i| i.exit_code)
            .unwrap_or(-1) as i32;

        Ok(SandboxExecResult {
            exit_code,
            stdout:      result.0,
            stderr:      result.1,
            duration_ms: start.elapsed().as_millis() as u64,
            changes:     vec![],
        })
    }

    /// Container-Layer committen → Image-Snapshot.
    async fn snapshot(&self, sandbox_id: Uuid) -> AppResult<String> {
        let name       = Self::container_name(sandbox_id);
        let snap_tag   = format!("devstudio-snap-{}", &sandbox_id.to_string()[..8]);
        let snap_image = format!("forgefabrik/devstudio-snapshot:{snap_tag}");

        self.docker
            .commit_container(
                CommitContainerOptions {
                    container: name,
                    repo:      "forgefabrik/devstudio-snapshot".to_string(),
                    tag:       snap_tag.clone(),
                    ..Default::default()
                },
                bollard::models::ContainerConfig::default(),
            )
            .await
            .map_err(|e| AppError::Sandbox(format!("snapshot commit: {e}")))?;

        tracing::debug!(sandbox_id = %sandbox_id, image = %snap_image, "DockerSandbox: snapshot");
        Ok(snap_image)
    }

    /// Neuen Container vom Snapshot-Image erstellen.
    async fn restore(&self, sandbox_id: Uuid, snapshot_id: &str) -> AppResult<()> {
        // Alten Container stoppen + entfernen
        let name = Self::container_name(sandbox_id);
        let _ = self.docker
            .stop_container(&name, Some(StopContainerOptions { t: 2 }))
            .await;
        let _ = self.docker
            .remove_container(&name, Some(RemoveContainerOptions { force: true, ..Default::default() }))
            .await;

        // Neuen Container vom Snapshot starten
        let options = CreateContainerOptions { name: &name, platform: None };
        let config  = Config::<String> {
            image:            Some(snapshot_id.to_owned()),
            cmd:              Some(vec!["sleep".into(), "3600".into()]),
            network_disabled: Some(true),
            host_config:      Some(bollard::models::HostConfig {
                cpu_quota:  Some(CPU_QUOTA),
                cpu_period: Some(CPU_PERIOD),
                memory:     Some(MEMORY_LIMIT),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| AppError::Sandbox(format!("restore create: {e}")))?;
        self.docker
            .start_container(&name, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| AppError::Sandbox(format!("restore start: {e}")))?;

        tracing::debug!(sandbox_id = %sandbox_id, snapshot = %snapshot_id, "DockerSandbox: restored");
        Ok(())
    }

    /// Container stoppen und entfernen.
    async fn destroy(&self, sandbox_id: Uuid) -> AppResult<()> {
        let name = Self::container_name(sandbox_id);
        let _ = self.docker
            .stop_container(&name, Some(StopContainerOptions { t: 5 }))
            .await;
        self.docker
            .remove_container(&name, Some(RemoveContainerOptions { force: true, ..Default::default() }))
            .await
            .map_err(|e| AppError::Sandbox(format!("remove container: {e}")))?;
        tracing::debug!(sandbox_id = %sandbox_id, "DockerSandbox: zerstört");
        Ok(())
    }

    /// `git diff` innerhalb des Containers ausführen.
    async fn diff(&self, sandbox_id: Uuid) -> AppResult<String> {
        let req = SandboxExecRequest {
            command:      "git".into(),
            args:         vec!["diff".into(), "--stat".into()],
            env:          HashMap::new(),
            timeout_secs: Some(10),
        };
        let result = self.execute(sandbox_id, req).await?;
        Ok(if result.exit_code == 0 { result.stdout } else { result.stderr })
    }
}
