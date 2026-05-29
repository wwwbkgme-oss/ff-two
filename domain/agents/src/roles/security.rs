use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

pub struct SecurityAgent { pub id: Uuid }
impl SecurityAgent { pub fn new() -> Self { Self { id: Uuid::new_v4() } } }
impl Default for SecurityAgent { fn default() -> Self { Self::new() } }

#[async_trait]
impl DevRolePlugin for SecurityAgent {
    fn role_name(&self) -> &'static str { "security" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if matches!(task.required_role, Some(AgentRole::Security)) {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Security, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        tracing::info!(task_id = %task.id, changes = exec.changes.len(), "security agent: scanning");
        let critical = exec.changes.iter()
            .flat_map(|c| c.content.lines().enumerate().map(move |(i, l)| (c.path.clone(), i + 1, l.to_string())))
            .filter(|(_, _, l)| l.contains("password") || l.contains("secret") || l.contains("SKIP_VERIFY"))
            .count();
        Ok(AgentOutput {
            summary:  format!("{critical} kritische Findings"),
            changes:  vec![],
            metadata: serde_json::json!({ "critical": critical, "passed": critical == 0, "agent_role": "security" }),
        })
    }

    async fn review(&self, change: &CodeChange) -> AppResult<Review> {
        let issues: Vec<_> = change.content.lines().enumerate()
            .filter(|(_, l)| l.contains("password") || l.contains("secret") || l.contains("eval("))
            .map(|(i, l)| format!("Zeile {}: {}", i + 1, l.trim()))
            .collect();
        let passed = issues.is_empty();
        Ok(Review { approved: passed, comments: issues, score: if passed { 95 } else { 20 } })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success) }
}
