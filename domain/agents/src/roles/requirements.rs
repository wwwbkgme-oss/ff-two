use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

pub struct RequirementsAgent { pub id: Uuid }
impl RequirementsAgent { pub fn new() -> Self { Self { id: Uuid::new_v4() } } }
impl Default for RequirementsAgent { fn default() -> Self { Self::new() } }

#[async_trait]
impl DevRolePlugin for RequirementsAgent {
    fn role_name(&self) -> &'static str { "requirements" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if task.required_role == Some(AgentRole::Requirements) || task.required_role.is_none() {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Requirements, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, _exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        tracing::info!(task_id = %task.id, "requirements agent: analysing");
        Ok(AgentOutput {
            summary:  format!("Requirements analysiert: {}", task.title),
            changes:  vec![],
            metadata: serde_json::json!({ "user_stories": [], "acceptance_criteria": [], "agent_role": "requirements" }),
        })
    }

    async fn review(&self, _change: &CodeChange) -> AppResult<Review> {
        Ok(Review { approved: true, comments: vec!["Requirements-Review: OK".into()], score: 90 })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success) }
}
