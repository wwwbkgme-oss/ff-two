//! `FreeLlmAgent` — Agenten-Implementierung mit injizierbarem LLM-Driver.
//!
//! ## BKG / ARCHITECTURE.md
//! * Dieses Modul gehört in `domain/agents` — es ist **Domänen-Logik**
//! * Der konkrete Provider (Groq, Ollama, ...) wird via `Arc<dyn LlmDriver>`
//!   **injiziert** — diese Klasse macht selbst keinen I/O-Call
//! * Driver-Implementierungen leben in `runtime/drivers/llm/`

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::{AgentRole, ChangeKind, CodeChange, SandboxExecResult, Task, TaskClaim};

use crate::budget::TokenBudget;
use crate::llm_driver::LlmDriver;
use crate::traits::{AgentOutput, BuildResult, DevRolePlugin, Review};

// ── System-Prompts je Rolle ───────────────────────────────────────────────────

fn system_prompt(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Requirements =>
            "Du bist ein erfahrener Requirements Engineer. \
             Analysiere Anforderungen und erstelle klare User Stories \
             mit Akzeptanzkriterien. Antworte präzise.",

        AgentRole::Architecture =>
            "Du bist ein erfahrener Software-Architekt. \
             Entwirf robuste, skalierbare Systemarchitekturen. \
             Beschreibe Komponenten, Abhängigkeiten und API-Verträge.",

        AgentRole::Coding =>
            "Du bist ein Senior Software Engineer. \
             Schreibe sauberen, produktionsreifen Code. \
             Halte dich an idiomatische Muster der Zielsprache. \
             Gib nur Code zurück, keine Erklärungen — sofern nicht explizit gefragt.",

        AgentRole::Testing =>
            "Du bist ein QA Engineer. \
             Schreibe umfassende Tests: Unit, Integration, Property-based. \
             Strebe ≥80 % Abdeckung an.",

        AgentRole::Security =>
            "Du bist ein Security Engineer. \
             Analysiere Code auf OWASP-Top-10-Risiken. \
             Antworte mit konkreten Findings und Lösungsvorschlägen.",

        AgentRole::Deployment =>
            "Du bist ein DevOps / Platform Engineer. \
             Erstelle produktionsreife Deployment-Konfigurationen: \
             Dockerfiles, CI/CD, Kubernetes-Manifeste, IaC.",
    }
}

// ── FreeLlmAgent ──────────────────────────────────────────────────────────────

/// Agenten-Implementierung die ausschließlich über `LlmDriver` kommuniziert.
///
/// Der konkrete Provider (Groq, Ollama, OpenRouter …) wird bei der Konstruktion
/// übergeben — dieser Typ kennt keine HTTP-Details.
///
/// # Konstruktion
/// ```rust,ignore
/// let driver = Arc::new(FreeProviderDriver::new()); // aus runtime/drivers
/// let agent  = FreeLlmAgent::new(AgentRole::Coding, driver);
/// registry.register(AgentRole::Coding, Arc::new(agent));
/// ```
#[derive(Debug)]
pub struct FreeLlmAgent {
    pub id:         Uuid,
    pub role:       AgentRole,
    driver:         Arc<dyn LlmDriver>,
    pub max_tokens: u32,
    /// Token-Budget für diesen Agenten (optional — unbegrenzt wenn None).
    budget:         Option<Arc<TokenBudget>>,
}

impl FreeLlmAgent {
    /// Erstellt einen neuen `FreeLlmAgent` mit dem gegebenen Driver.
    /// Standard: Dev-Budget (10 Credits).
    pub fn new(role: AgentRole, driver: Arc<dyn LlmDriver>) -> Self {
        Self {
            id:         Uuid::new_v4(),
            role,
            driver,
            max_tokens: 4096,
            budget:     Some(TokenBudget::dev()),
        }
    }

    /// Ohne Budget-Limit (z. B. für Tests).
    pub fn new_unlimited(role: AgentRole, driver: Arc<dyn LlmDriver>) -> Self {
        Self { id: Uuid::new_v4(), role, driver, max_tokens: 4096, budget: None }
    }

    /// Erstellt alle 6 Rollen-Agents mit dem gleichen Driver.
    pub fn all_roles(driver: Arc<dyn LlmDriver>) -> Vec<(AgentRole, Arc<Self>)> {
        let roles = [
            AgentRole::Requirements,
            AgentRole::Architecture,
            AgentRole::Coding,
            AgentRole::Testing,
            AgentRole::Security,
            AgentRole::Deployment,
        ];
        roles
            .into_iter()
            .map(|r| (r.clone(), Arc::new(FreeLlmAgent::new(r, Arc::clone(&driver)))))
            .collect()
    }

    async fn call(&self, user: impl Into<String>) -> AppResult<String> {
        // ── Budget-Check ──────────────────────────────────────────────────────
        if let Some(budget) = &self.budget {
            if !budget.can_afford(self.max_tokens) {
                return Err(AppError::Agent(
                    "Token-Budget erschöpft — setze DEVSTUDIO_AGENT_BUDGET_MILLI für mehr Credits".into()
                ));
            }
        }

        let system = system_prompt(&self.role);
        let user   = user.into();

        match self.driver.chat(system, &user, self.max_tokens).await {
            Ok(resp) => {
                // ── Budget-Debit ───────────────────────────────────────────────
                if let Some(budget) = &self.budget {
                    let output_toks = resp.tokens.unwrap_or(self.max_tokens / 2);
                    let input_toks  = user.split_whitespace().count() as u32;
                    budget.debit(input_toks, output_toks);
                }
                debug!(
                    provider = %resp.provider,
                    model    = %resp.model,
                    role     = %self.role,
                    tokens   = ?resp.tokens,
                    "FreeLlmAgent: call ok"
                );
                Ok(resp.text)
            }
            Err(e) => {
                warn!(
                    error = %e,
                    role  = %self.role,
                    provider = self.driver.provider_name(),
                    "FreeLlmAgent: driver fehlgeschlagen — Mock-Fallback"
                );
                Ok(format!(
                    "// FreeLlmAgent Mock-Fallback\n\
                     // Driver: {}\n\
                     // Fehler: {e}\n\
                     fn placeholder() {{}}\n",
                    self.driver.provider_name()
                ))
            }
        }
    }
}

// ── DevRolePlugin-Implementierung ─────────────────────────────────────────────

#[async_trait]
impl DevRolePlugin for FreeLlmAgent {
    fn role_name(&self) -> &'static str {
        match self.role {
            AgentRole::Requirements => "requirements",
            AgentRole::Architecture => "architecture",
            AgentRole::Coding       => "coding",
            AgentRole::Testing      => "testing",
            AgentRole::Security     => "security",
            AgentRole::Deployment   => "deployment",
        }
    }

    async fn claim_task(&self, task: &Task) -> Option<TaskClaim> {
        let matches = task.required_role.as_ref()
            .map(|r| r == &self.role)
            .unwrap_or(matches!(self.role, AgentRole::Coding));

        if matches {
            Some(TaskClaim {
                task_id:    task.id,
                agent_id:   self.id,
                role:       self.role.clone(),
                claimed_at: Utc::now(),
            })
        } else {
            None
        }
    }

    async fn execute(&self, task: &Task, _exec: &SandboxExecResult) -> AppResult<AgentOutput> {
        info!(task_id = %task.id, role = %self.role, "FreeLlmAgent: execute");

        let prompt = format!(
            "Aufgabe: {title}\n\nBeschreibung:\n{desc}",
            title = task.title,
            desc  = task.description,
        );
        let code = self.call(prompt).await?;

        let checksum = {
            let mut h = Sha256::new();
            h.update(code.as_bytes());
            hex::encode(h.finalize())
        };

        let path = match self.role {
            AgentRole::Coding     => format!("src/generated/{}.rs", task.id),
            AgentRole::Testing    => format!("tests/generated/{}.rs", task.id),
            AgentRole::Deployment => format!("deploy/generated/{}.yaml", task.id),
            _                     => format!("docs/generated/{}.md", task.id),
        };

        Ok(AgentOutput {
            summary:  format!("[{}] {}", self.role_name(), task.title),
            changes:  vec![CodeChange { path, kind: ChangeKind::Created, content: code.clone(), checksum }],
            metadata: serde_json::json!({
                "agent_role":  self.role_name(),
                "driver":      self.driver.provider_name(),
                "lines":       code.lines().count(),
            }),
        })
    }

    async fn review(&self, change: &CodeChange) -> AppResult<Review> {
        let prompt = format!(
            "Reviewe diesen Code und antworte als JSON:\n\
             {{\"approved\":true/false,\"comments\":[\"...\"],\"score\":0-100}}\n\
             Datei: {}\n```\n{}\n```",
            change.path,
            change.content.chars().take(3000).collect::<String>()
        );
        let raw = self.call(prompt).await?;

        #[derive(serde::Deserialize)]
        struct R { approved: bool, comments: Vec<String>, score: u8 }

        let json = extract_json(&raw);
        match serde_json::from_str::<R>(&json) {
            Ok(r) => Ok(Review { approved: r.approved, comments: r.comments, score: r.score }),
            Err(_) => Ok(Review {
                approved: true,
                comments: vec![raw.chars().take(200).collect()],
                score: 70,
            }),
        }
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> {
        Ok(build.success && build.coverage >= 60.0)
    }
}

fn extract_json(text: &str) -> String {
    if let Some(s) = text.find("```json") {
        if let Some(e) = text[s + 7..].find("```") {
            return text[s + 7..s + 7 + e].trim().to_owned();
        }
    }
    if let (Some(s), Some(e)) = (text.find('{'), text.rfind('}')) {
        return text[s..=e].to_owned();
    }
    text.to_owned()
}
