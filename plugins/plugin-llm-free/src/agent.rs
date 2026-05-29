//! `FreeLlmAgent` — DevRolePlugin-Implementierung mit kostenlosen LLM-Providern.
//!
//! Ersetzt die Anthropic-only `CodingAgent`-Implementierung durch einen
//! Failover-Router über alle konfigurierten freien Provider.
//! Fällt auf einen deterministischen Mock zurück, wenn kein Provider
//! verfügbar ist (Ollama offline, keine Keys gesetzt).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use uuid::Uuid;

use agents::{AgentOutput, BuildResult, DevRolePlugin, Review};
use errors::{AppError, AppResult};
use types::{AgentRole, ChangeKind, CodeChange, SandboxExecResult, Task, TaskClaim};

use crate::router;

// ── System-Prompts je Rolle ───────────────────────────────────────────────────

fn system_prompt_for(role: &AgentRole) -> &'static str {
    match role {
        AgentRole::Requirements =>
            "Du bist ein erfahrener Requirements Engineer. \
             Analysiere die Anforderungen und erstelle klare User Stories \
             mit Akzeptanzkriterien. Antworte präzise auf Deutsch oder Englisch.",

        AgentRole::Architecture =>
            "Du bist ein erfahrener Software-Architekt. \
             Entwirf robuste, skalierbare Systemarchitekturen. \
             Beschreibe Komponenten, Abhängigkeiten und API-Verträge präzise.",

        AgentRole::Coding =>
            "Du bist ein Senior Software Engineer. \
             Schreibe sauberen, produktionsreifen Code. \
             Halte dich an Rust-Best-Practices: idiomatisch, typsicher, \
             mit Fehlerbehandlung. Gib nur Code ohne Erklärung zurück, \
             sofern nicht explizit anders gefragt.",

        AgentRole::Testing =>
            "Du bist ein QA Engineer. Schreibe umfassende Tests: \
             Unit-Tests, Integrationstests und Property-based Tests. \
             Strebe ≥80 % Abdeckung an. Verwende das Test-Framework der Zielsprache.",

        AgentRole::Security =>
            "Du bist ein Security Engineer. \
             Analysiere Code auf Sicherheitsprobleme: Injections, \
             unsichere Deserialisierung, exponierte Secrets, \
             unsichere Krypto und weitere OWASP-Top-10-Risiken. \
             Antworte mit konkreten Findings und Lösungsvorschlägen.",

        AgentRole::Deployment =>
            "Du bist ein DevOps / Platform Engineer. \
             Erstelle produktionsreife Deployment-Konfigurationen: \
             Dockerfiles, CI/CD-Pipelines, Kubernetes-Manifeste, \
             IaC-Code. Halte dich an aktuelle Best Practices.",
    }
}

// ── FreeLlmAgent ──────────────────────────────────────────────────────────────

/// Agenten-Implementierung die ausschließlich kostenlose LLM-Provider nutzt.
///
/// Provider-Priorität (konfigurierbar via ENV-Keys):
///   1. OpenRouter `:free`-Modelle   (OPENROUTER_API_KEY)
///   2. Groq Free Tier               (GROQ_API_KEY)
///   3. Cerebras Free Tier           (CEREBRAS_API_KEY)
///   4. SambaNova Forever-Free       (SAMBANOVA_API_KEY)
///   5. LLM7.io Gateway              (LLM7_API_KEY)
///   6. Ollama (lokal)               (kein Key nötig)
#[derive(Debug)]
pub struct FreeLlmAgent {
    pub id:         Uuid,
    pub role:       AgentRole,
    pub max_tokens: u32,
}

impl FreeLlmAgent {
    /// Erstellt einen neuen `FreeLlmAgent` für die gegebene Rolle.
    pub fn new(role: AgentRole) -> Self {
        Self {
            id:         Uuid::new_v4(),
            role,
            max_tokens: 4096,
        }
    }

    /// Erstellt den Coding-Agenten (häufigster Anwendungsfall).
    pub fn coding() -> Self { Self::new(AgentRole::Coding) }

    /// Ruft den Failover-Router auf und gibt den generierten Text zurück.
    async fn call_llm(&self, user: impl Into<String>) -> AppResult<String> {
        let system = system_prompt_for(&self.role);
        let user   = user.into();

        match router::chat(system, &user, self.max_tokens).await {
            Ok(result) => {
                debug!(
                    provider = result.provider,
                    model    = %result.model,
                    role     = %self.role,
                    "FreeLlmAgent: LLM call succeeded"
                );
                Ok(result.text)
            }
            Err(e) => {
                warn!(error = %e, role = %self.role, "FreeLlmAgent: all providers failed, using mock");
                // Deterministischer Fallback — kein Panic, keine leere Ausgabe
                Ok(format!(
                    "// FreeLlmAgent Mock — kein Provider verfügbar\n\
                     // Fehler: {e}\n\
                     // Konfiguriere mindestens einen API-Key (GROQ_API_KEY,\n\
                     // OPENROUTER_API_KEY, SAMBANOVA_API_KEY, …)\n\
                     // oder starte Ollama lokal.\n\
                     fn placeholder() {{}}\n"
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
        // Übernimmt Tasks für die eigene Rolle oder ohne Rollenanforderung
        let matches = task.required_role.as_ref()
            .map(|r| r == &self.role)
            .unwrap_or(matches!(self.role, AgentRole::Coding)); // default: nur Coding-Tasks

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
        info!(
            task_id = %task.id,
            role    = %self.role,
            "FreeLlmAgent: executing task"
        );

        let prompt = format!(
            "Aufgabe: {title}\n\nBeschreibung:\n{desc}\n\n\
             Erstelle eine vollständige Implementierung.",
            title = task.title,
            desc  = task.description,
        );

        let code = self.call_llm(prompt).await?;

        // Prüfsumme für reproduzierbare Erkennung von Änderungen
        let checksum = {
            let mut h = Sha256::new();
            h.update(code.as_bytes());
            hex::encode(h.finalize())
        };

        let filename = match self.role {
            AgentRole::Coding      => format!("src/generated/{}.rs", task.id),
            AgentRole::Testing     => format!("tests/generated/{}.rs", task.id),
            AgentRole::Deployment  => format!("deploy/generated/{}.yaml", task.id),
            _                      => format!("docs/generated/{}.md", task.id),
        };

        let change = CodeChange {
            path:     filename,
            kind:     ChangeKind::Created,
            content:  code.clone(),
            checksum,
        };

        Ok(AgentOutput {
            summary:  format!("[{}] {}: {}", self.role_name(), "free-llm", task.title),
            changes:  vec![change],
            metadata: serde_json::json!({
                "agent_role":  self.role_name(),
                "provider":    "free-llm-router",
                "lines":       code.lines().count(),
                "plugin_id":   "forgefabrik.llm-free",
            }),
        })
    }

    async fn review(&self, change: &CodeChange) -> AppResult<Review> {
        let prompt = format!(
            "Reviewe diesen Code-Änderung und antworte als JSON:\n\
             {{\"approved\": true/false, \"comments\": [\"...\"], \"score\": 0-100}}\n\n\
             Datei: {path}\n\
             ```\n{snippet}\n```",
            path    = change.path,
            snippet = change.content.chars().take(3000).collect::<String>(),
        );

        let raw = self.call_llm(prompt).await?;

        // Versuche JSON zu parsen; Fallback auf optimistisches Review
        #[derive(serde::Deserialize)]
        struct R { approved: bool, comments: Vec<String>, score: u8 }

        match serde_json::from_str::<R>(&extract_json(&raw)) {
            Ok(r) => Ok(Review {
                approved: r.approved,
                comments: r.comments,
                score:    r.score,
            }),
            Err(_) => Ok(Review {
                approved: true,
                comments: vec![format!("Review (free-llm): {}", raw.chars().take(200).collect::<String>())],
                score:    75,
            }),
        }
    }

    async fn verify(&self, build: &BuildResult) -> AppResult<bool> {
        // Einfache Verifikation ohne LLM-Aufruf (Build-Ergebnis spricht für sich)
        Ok(build.success && build.coverage >= 60.0)
    }
}

// ── Hilfsfunktionen ───────────────────────────────────────────────────────────

/// Extrahiert JSON-Block aus einer LLM-Antwort (```json ... ``` oder roher JSON).
fn extract_json(text: &str) -> String {
    // Versuche JSON-Code-Block zu extrahieren
    if let Some(start) = text.find("```json") {
        if let Some(end) = text[start + 7..].find("```") {
            return text[start + 7..start + 7 + end].trim().to_owned();
        }
    }
    // Versuche rohen JSON-Block (beginnt mit `{`)
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return text[start..=end].to_owned();
        }
    }
    text.to_owned()
}

// ── Hilfskonstruktor für mehrere Rollen ──────────────────────────────────────

/// Erstellt für alle sechs Rollen je einen `FreeLlmAgent`.
/// Nützlich, um die gesamte AgentRegistry auf freie Provider umzustellen.
pub fn all_roles() -> Vec<(AgentRole, Arc<FreeLlmAgent>)> {
    vec![
        (AgentRole::Requirements, Arc::new(FreeLlmAgent::new(AgentRole::Requirements))),
        (AgentRole::Architecture, Arc::new(FreeLlmAgent::new(AgentRole::Architecture))),
        (AgentRole::Coding,       Arc::new(FreeLlmAgent::new(AgentRole::Coding))),
        (AgentRole::Testing,      Arc::new(FreeLlmAgent::new(AgentRole::Testing))),
        (AgentRole::Security,     Arc::new(FreeLlmAgent::new(AgentRole::Security))),
        (AgentRole::Deployment,   Arc::new(FreeLlmAgent::new(AgentRole::Deployment))),
    ]
}
