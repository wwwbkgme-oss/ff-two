use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

pub struct ArchitectureAgent { pub id: Uuid }
impl ArchitectureAgent { pub fn new() -> Self { Self { id: Uuid::new_v4() } } }
impl Default for ArchitectureAgent { fn default() -> Self { Self::new() } }

#[async_trait]
impl DevRolePlugin for ArchitectureAgent {
    fn role_name(&self) -> &'static str { "architecture" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if matches!(task.required_role, Some(AgentRole::Architecture)) {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Architecture, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, _exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        tracing::info!(task_id = %task.id, "architecture agent: designing");
        Ok(AgentOutput {
            summary:  format!("Architektur entworfen: {}", task.title),
            changes:  vec![],
            metadata: serde_json::json!({ "components": [], "api_contracts": [], "agent_role": "architecture" }),
        })
    }

    async fn review(&self, _change: &CodeChange) -> AppResult<Review> {
        Ok(Review { approved: true, comments: vec!["Architektur-Review: solides Design".into()], score: 85 })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success) }
}
