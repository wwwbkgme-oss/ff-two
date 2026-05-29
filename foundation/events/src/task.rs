use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::{AgentRole, TaskPriority, TaskStatus};

/// Lebenszyklus-Events einer Task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TaskEvent {
    Created   { task_id: Uuid, project_id: Uuid, title: String, priority: TaskPriority },
    Queued    { task_id: Uuid },
    Claimed   { task_id: Uuid, agent_id: Uuid, role: AgentRole },
    Started   { task_id: Uuid, sandbox_id: Uuid },
    Completed { task_id: Uuid, output: serde_json::Value },
    Failed    { task_id: Uuid, reason: String },
    Cancelled { task_id: Uuid },
    VoteAdded { task_id: Uuid, voter: String, approved: bool },
    ConsensusMet { task_id: Uuid, threshold: usize },
}
