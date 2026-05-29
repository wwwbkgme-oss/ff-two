use async_trait::async_trait;
use uuid::Uuid;

use errors::AppResult;
use types::{Sandbox, SandboxExecRequest, SandboxExecResult};

/// Abstraktes Sandbox-Interface — Backends: Local, Docker, …
#[async_trait]
pub trait SandboxManager: Send + Sync + 'static {
    async fn create(&self, project_id: Uuid)                              -> AppResult<Sandbox>;
    async fn execute(&self, id: Uuid, req: SandboxExecRequest)            -> AppResult<SandboxExecResult>;
    async fn snapshot(&self, id: Uuid)                                    -> AppResult<String>;
    async fn restore(&self, id: Uuid, snapshot_id: &str)                  -> AppResult<()>;
    async fn destroy(&self, id: Uuid)                                     -> AppResult<()>;
    async fn diff(&self, id: Uuid)                                        -> AppResult<String>;
}
