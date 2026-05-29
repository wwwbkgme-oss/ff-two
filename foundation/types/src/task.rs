use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::AgentRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority { Low = 0, Medium = 1, High = 2, Critical = 3 }
impl Default for TaskPriority { fn default() -> Self { Self::Medium } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending, Queued, Claimed, InProgress, Review,
    Testing, SecurityScan, Completed, Failed, Cancelled,
}
impl Default for TaskStatus { fn default() -> Self { Self::Pending } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id:            Uuid,
    pub project_id:    Uuid,
    pub title:         String,
    pub description:   String,
    pub priority:      TaskPriority,
    pub status:        TaskStatus,
    pub required_role: Option<AgentRole>,
    pub claimed_by:    Option<Uuid>,
    pub sandbox_id:    Option<Uuid>,
    pub output:        Option<serde_json::Value>,
    pub error:         Option<String>,
    pub notes:         Vec<String>,
    /// Konsens-Votes: agent_id → approved.
    pub consensus:     HashMap<String, bool>,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
    pub completed_at:  Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(
        project_id:  Uuid,
        title:       impl Into<String>,
        description: impl Into<String>,
        priority:    TaskPriority,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(), project_id,
            title: title.into(), description: description.into(),
            priority, status: TaskStatus::Pending, required_role: None,
            claimed_by: None, sandbox_id: None, output: None, error: None,
            notes: Vec::new(), consensus: HashMap::new(),
            created_at: now, updated_at: now, completed_at: None,
        }
    }

    /// Gibt `true` zurück, wenn mindestens `threshold` positive Votes vorliegen.
    pub fn has_consensus(&self, threshold: usize) -> bool {
        self.consensus.values().filter(|&&v| v).count() >= threshold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskClaim {
    pub task_id:    Uuid,
    pub agent_id:   Uuid,
    pub role:       AgentRole,
    pub claimed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title:         String,
    pub description:   Option<String>,
    pub priority:      Option<TaskPriority>,
    pub required_role: Option<AgentRole>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskRequest { pub agent_id: Uuid }
