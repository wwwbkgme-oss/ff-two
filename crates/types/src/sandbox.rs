use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxStatus { Creating, Ready, Executing, Paused, Destroyed, Error }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind { Created, Modified, Deleted, Renamed }

/// A single file change produced by an agent inside a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub path:     String,
    pub kind:     ChangeKind,
    /// Full new content or unified diff depending on `kind`.
    pub content:  String,
    pub checksum: String,
}

/// An isolated workspace assigned to one agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id:              Uuid,
    pub project_id:      Uuid,
    pub agent_id:        Option<Uuid>,
    pub status:          SandboxStatus,
    /// Container or process ID in the underlying runtime.
    pub runtime_id:      Option<String>,
    pub work_dir:        String,
    /// Snapshot ID for rollback capability.
    pub snapshot_id:     Option<String>,
    pub pending_changes: Vec<CodeChange>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

impl Sandbox {
    pub fn new(project_id: Uuid, work_dir: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(), project_id, agent_id: None,
            status: SandboxStatus::Creating, runtime_id: None,
            work_dir: work_dir.into(), snapshot_id: None,
            pending_changes: Vec::new(), created_at: now, updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecRequest {
    pub command:      String,
    pub args:         Vec<String>,
    pub env:          HashMap<String, String>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecResult {
    pub exit_code:   i32,
    pub stdout:      String,
    pub stderr:      String,
    pub duration_ms: u64,
    pub changes:     Vec<CodeChange>,
}
