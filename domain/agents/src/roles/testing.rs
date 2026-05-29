use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

pub struct TestingAgent { pub id: Uuid }
impl TestingAgent { pub fn new() -> Self { Self { id: Uuid::new_v4() } } }
impl Default for TestingAgent { fn default() -> Self { Self::new() } }

#[async_trait]
impl DevRolePlugin for TestingAgent {
    fn role_name(&self) -> &'static str { "testing" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if matches!(task.required_role, Some(AgentRole::Testing)) {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Testing, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        let passed = exec.exit_code == 0;
        let count  = exec.stdout.lines().filter(|l| l.contains("test ") && (l.contains("ok") || l.contains("PASSED"))).count();
        tracing::info!(task_id = %task.id, tests = count, passed, "testing agent");
        Ok(AgentOutput {
            summary:  format!("{count} Tests, {}", if passed { "alle bestanden" } else { "einige fehlgeschlagen" }),
            changes:  vec![],
            metadata: serde_json::json!({ "tests_run": count, "passed": passed, "agent_role": "testing" }),
        })
    }

    async fn review(&self, change: &CodeChange) -> AppResult<Review> {
        let has_tests = change.content.contains("#[test]") || change.content.contains("def test_")
                     || change.content.contains("it(\"")   || change.content.contains("describe(\"");
        Ok(Review {
            approved: has_tests,
            comments: if has_tests { vec!["Tests gefunden.".into()] } else { vec!["Keine Tests — bitte Coverage hinzufügen.".into()] },
            score:    if has_tests { 90 } else { 30 },
        })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success && build.coverage >= 80.0) }
}
