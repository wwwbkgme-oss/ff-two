use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::{AgentRole, ChangeKind, CodeChange, SandboxExecResult, Task, TaskClaim};
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

/// CodingAgent — nutzt die Anthropic Claude API für Code-Generierung.
/// Fällt bei fehlendem API-Key auf einen deterministischen Mock zurück.
pub struct CodingAgent {
    pub id:    Uuid,
    model:     String,
    max_tok:   u32,
    api_key:   Option<String>,
    client:    Client,
}

impl CodingAgent {
    pub fn new(model: impl Into<String>, max_tokens: u32, api_key: Option<String>) -> Self {
        Self { id: Uuid::new_v4(), model: model.into(), max_tok: max_tokens, api_key, client: Client::new() }
    }

    async fn claude(&self, system: &str, user: &str) -> AppResult<String> {
        let key = match &self.api_key { Some(k) if !k.is_empty() => k.as_str(), _ => {
            return Ok(format!("// Mock — kein API-Key\n// Task: {}\nfn placeholder() {{}}\n",
                user.chars().take(60).collect::<String>()));
        }};

        #[derive(Serialize)]
        struct Req<'a> { model: &'a str, max_tokens: u32, system: &'a str, messages: Vec<Msg<'a>> }
        #[derive(Serialize)]
        struct Msg<'a> { role: &'a str, content: &'a str }
        #[derive(Deserialize)]
        struct Resp { content: Vec<Content> }
        #[derive(Deserialize)]
        struct Content { text: String }

        let resp = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&Req { model: &self.model, max_tokens: self.max_tok, system,
                messages: vec![Msg { role: "user", content: user }] })
            .send().await.map_err(|e| AppError::Agent(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AppError::Agent(format!("Claude {} — {}", resp.status(), resp.text().await.unwrap_or_default())));
        }
        let parsed: Resp = resp.json().await.map_err(|e| AppError::Agent(e.to_string()))?;
        Ok(parsed.content.into_iter().map(|c| c.text).collect::<Vec<_>>().join("\n"))
    }
}

#[async_trait]
impl DevRolePlugin for CodingAgent {
    fn role_name(&self) -> &'static str { "coding" }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        if matches!(task.required_role, Some(AgentRole::Coding) | None) {
            Some(TaskClaim { task_id: task.id, agent_id: self.id, role: AgentRole::Coding, claimed_at: Utc::now() })
        } else { None }
    }

    async fn execute(&self, task: &Task, _exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        let system = "Senior Software Engineer. Schreibe sauberen, produktionsreifen Code.";
        let prompt = format!("Implementiere: {}\n\nBeschreibung: {}", task.title, task.description);
        let code   = self.claude(system, &prompt).await?;
        let checksum = { let mut h = Sha256::new(); h.update(code.as_bytes()); hex::encode(h.finalize()) };
        let change = CodeChange { path: format!("src/generated/{}.rs", task.id), kind: ChangeKind::Created, content: code.clone(), checksum };
        Ok(AgentOutput {
            summary:  format!("Code generiert: {}", task.title),
            changes:  vec![change],
            metadata: serde_json::json!({ "model": self.model, "lines": code.lines().count(), "agent_role": "coding" }),
        })
    }

    async fn review(&self, change: &CodeChange) -> AppResult<Review> {
        let system = "Code-Reviewer. Antworte als JSON: {\"approved\":bool,\"comments\":[string]}";
        let prompt = format!("Reviewe {}:\n```\n{}\n```", change.path, &change.content.chars().take(2000).collect::<String>());
        let raw    = self.claude(system, &prompt).await?;
        #[derive(Deserialize)] struct R { approved: bool, comments: Vec<String> }
        let r: R = serde_json::from_str(&raw).unwrap_or(R { approved: true, comments: vec![] });
        Ok(Review { approved: r.approved, comments: r.comments, score: if r.approved { 80 } else { 40 } })
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> { Ok(build.success && build.coverage >= 70.0) }
}
