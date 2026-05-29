use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::SandboxStatus;

/// Lebenszyklus-Events einer Sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SandboxEvent {
    Created   { sandbox_id: Uuid, project_id: Uuid, work_dir: String },
    Ready     { sandbox_id: Uuid },
    Executing { sandbox_id: Uuid, command: String },
    Snapshot  { sandbox_id: Uuid, snapshot_id: String },
    Restored  { sandbox_id: Uuid, snapshot_id: String },
    Destroyed { sandbox_id: Uuid },
    Error     { sandbox_id: Uuid, reason: String },
}
