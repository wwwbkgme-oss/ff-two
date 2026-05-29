use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::{AgentRole, AgentStatus};

/// Lebenszyklus-Events eines Agenten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AgentEvent {
    Spawned     { agent_id: Uuid, role: AgentRole, model: String },
    TaskClaimed { agent_id: Uuid, task_id: Uuid },
    TaskStarted { agent_id: Uuid, task_id: Uuid },
    TaskDone    { agent_id: Uuid, task_id: Uuid, success: bool },
    ReviewDone  { agent_id: Uuid, task_id: Uuid, approved: bool },
    StatusChanged { agent_id: Uuid, new_status: AgentStatus },
    Offline     { agent_id: Uuid },
}
