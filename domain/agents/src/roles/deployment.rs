use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

pub struct DeploymentAgent { pub id: Uuid }
impl DeploymentAgent { pub fn new() -> Self { Self { id: Uuid::new_v4() } } }
impl Default for DeploymentAgent { fn default() -> Self { Self::new() } }

#[async_trait]
impl DevRolePlugin for DeploymentAgent {
    fn role_name(&self) -> &'static str { "deployment" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if matches!(task.required_role, Some(AgentRole::Deployment)) {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Deployment, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        tracing::info!(task_id = %task.id, "deployment agent: preparing");
        Ok(AgentOutput {
            summary:  format!("Deployment vorbereitet: {}", task.title),
            changes:  vec![],
            metadata: serde_json::json!({ "exit_code": exec.exit_code, "agent_role": "deployment" }),
        })
    }

    async fn review(&self, _change: &CodeChange) -> AppResult<Review> {
        Ok(Review { approved: true, comments: vec!["Deployment-Review: bereit.".into()], score: 88 })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success) }
}
