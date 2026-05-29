use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Spezialisierung eines Agenten innerhalb des Development Collective.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRole {
    Requirements, Architecture, Coding, Testing, Security, Deployment,
}
impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Requirements => "requirements", Self::Architecture => "architecture",
            Self::Coding       => "coding",       Self::Testing      => "testing",
            Self::Security     => "security",     Self::Deployment   => "deployment",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus { Idle, Working, Reviewing, Blocked, Offline }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id:              Uuid,
    pub name:            String,
    pub role:            AgentRole,
    pub status:          AgentStatus,
    pub current_task:    Option<Uuid>,
    pub tasks_completed: u64,
    pub model:           String,
    pub created_at:      DateTime<Utc>,
    pub last_seen:       DateTime<Utc>,
}

impl Agent {
    pub fn new(name: impl Into<String>, role: AgentRole, model: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(), name: name.into(), role,
            status: AgentStatus::Idle, current_task: None, tasks_completed: 0,
            model: model.into(), created_at: now, last_seen: now,
        }
    }
}
