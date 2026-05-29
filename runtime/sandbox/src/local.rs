use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use config::SandboxSettings;
use errors::{AppError, AppResult};
use types::{Sandbox, SandboxExecRequest, SandboxExecResult, SandboxStatus};

use super::manager::SandboxManager;

/// Prozessbasierter Sandbox-Manager — nutzt Tmpdir + tokio::process.
/// Für stärkere Isolation den DockerSandboxManager verwenden.
pub struct LocalSandboxManager {
    cfg:       SandboxSettings,
    sandboxes: Mutex<HashMap<Uuid, Sandbox>>,
}

impl std::fmt::Debug for LocalSandboxManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSandboxManager").finish()
    }
}

impl LocalSandboxManager {
    pub fn new(cfg: SandboxSettings) -> Self {
        Self { cfg, sandboxes: Mutex::new(HashMap::new()) }
    }
    fn dir(&self, id: Uuid) -> PathBuf { PathBuf::from(&self.cfg.work_dir).join(id.to_string()) }
}

fn lock_err<T>(e: std::sync::PoisonError<T>) -> AppError {
    AppError::Sandbox(format!("mutex poisoned: {e}"))
}

#[async_trait]
impl SandboxManager for LocalSandboxManager {
    async fn create(&self, project_id: Uuid) -> AppResult<Sandbox> {
        let mut sb = Sandbox::new(project_id, "");
        let dir    = self.dir(sb.id);
        tokio::fs::create_dir_all(&dir).await.map_err(|e| AppError::Sandbox(e.to_string()))?;
        sb.work_dir = dir.to_string_lossy().into_owned();
        sb.status   = SandboxStatus::Ready;
        self.sandboxes.lock().map_err(lock_err)?.insert(sb.id, sb.clone());
        tracing::debug!(id = %sb.id, dir = %sb.work_dir, "sandbox created");
        Ok(sb)
    }

    async fn execute(&self, sandbox_id: Uuid, req: SandboxExecRequest) -> AppResult<SandboxExecResult> {
        let work_dir = self.sandboxes.lock().map_err(lock_err)?
            .get(&sandbox_id).map(|s| s.work_dir.clone())
            .ok_or_else(|| AppError::not_found("sandbox", sandbox_id))?;

        let timeout = Duration::from_secs(req.timeout_secs.unwrap_or(self.cfg.timeout_secs));
        let start   = Instant::now();
        let mut cmd = tokio::process::Command::new(&req.command);
        cmd.args(&req.args).current_dir(&work_dir).envs(&req.env).kill_on_drop(true);
        let out = tokio::time::timeout(timeout, cmd.output()).await
            .map_err(|_| AppError::Sandbox("execution timed out".into()))?
            .map_err(|e| AppError::Sandbox(format!("spawn: {e}")))?;
        Ok(SandboxExecResult {
            exit_code:   out.status.code().unwrap_or(-1),
            stdout:      String::from_utf8_lossy(&out.stdout).into(),
            stderr:      String::from_utf8_lossy(&out.stderr).into(),
            duration_ms: start.elapsed().as_millis() as u64,
            changes:     vec![],
        })
    }

    async fn snapshot(&self, sandbox_id: Uuid) -> AppResult<String> {
        let dir = self.sandboxes.lock().map_err(lock_err)?
            .get(&sandbox_id).map(|s| s.work_dir.clone())
            .ok_or_else(|| AppError::not_found("sandbox", sandbox_id))?;
        let mut h = Sha256::new();
        h.update(dir.as_bytes());
        h.update(Utc::now().timestamp_millis().to_le_bytes());
        Ok(hex::encode(h.finalize()))
    }

    async fn restore(&self, sandbox_id: Uuid, _snapshot_id: &str) -> AppResult<()> {
        tracing::warn!(id = %sandbox_id, "local sandbox restore ist ein No-Op");
        Ok(())
    }

    async fn destroy(&self, sandbox_id: Uuid) -> AppResult<()> {
        let dir = self.dir(sandbox_id);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await.map_err(|e| AppError::Sandbox(e.to_string()))?;
        }
        self.sandboxes.lock().map_err(lock_err)?.remove(&sandbox_id);
        Ok(())
    }

    async fn diff(&self, sandbox_id: Uuid) -> AppResult<String> {
        Ok(format!("# diff sandbox {sandbox_id} (local mode — kein Git-Tracking)"))
    }
}
