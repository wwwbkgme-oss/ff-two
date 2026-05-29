use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use errors::AppResult;
use types::{AgentRole, CodeChange, SandboxExecResult, Task, TaskClaim};

// ── Output-Typen ──────────────────────────────────────────────────────────

/// Ergebnis einer abgeschlossenen Aufgabe.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub summary:  String,
    pub changes:  Vec<CodeChange>,
    pub metadata: serde_json::Value,
}

/// Code-Review-Ergebnis eines Agenten.
#[derive(Debug, Clone)]
pub struct Review {
    pub approved: bool,
    pub comments: Vec<String>,
    pub score:    u8,
}

/// Build-Ergebnis für die `verify`-Phase.
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub success:  bool,
    pub output:   String,
    pub coverage: f32,
}

// ── Core-Traits (aus der Spezifikation) ───────────────────────────────────

/// Kern-Plugin-Interface für eine Agentenrolle — entspricht dem Spec-Trait.
#[async_trait]
pub trait DevRolePlugin: Send + Sync + 'static {
    fn role_name(&self) -> &'static str;
    async fn claim_task(&self, task: &Task)                                     -> Option<TaskClaim>;
    async fn execute(&self, task: &Task, exec: &SandboxExecResult)              -> AppResult<AgentOutput>;
    async fn review(&self, change: &CodeChange)                                 -> AppResult<Review>;
    async fn verify(&self, build: &BuildResult)                                 -> AppResult<bool>;
}

/// Welt-Visualisierungs-Plugin-Interface — ebenfalls aus der Spezifikation.
pub trait WorldVisualizationPlugin: Send + Sync + 'static {
    fn code_to_voxels(&self, path: &str, content: &str) -> Vec<types::VoxelBlock>;
    fn biome_for_tech(&self, tech: &str)                 -> types::BiomeType;
}
