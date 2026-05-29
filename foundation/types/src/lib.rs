//! `foundation/types` — Single Source of Truth für alle Domain-Datentypen.
//!
//! ## BKG-Regeln
//! - Kein HTTP, keine DB, kein Netzwerk
//! - Keine Businesslogik, keine State-Mutation
//! - Nur Typdefinitionen, IDs, Datenstrukturen
//!
//! ## forge-core SYNC_CONTRACT v0.1
//! Kanonische Typen: `WorldTick`, `TickContext`, `DeterministicRng`,
//! `FreeProvider`, `AgentKind`, `FfPluginCtx`, `export_forgefabrik_plugin!`

pub mod agent;
pub mod canonical;
pub mod deployment;
pub mod forge_plugin;
pub mod project;
pub mod sandbox;
pub mod task;
pub mod tick;
pub mod world;

// Flache Re-Exports für ergonomischen Import: `use types::Project;`
pub use agent::{Agent, AgentRole, AgentStatus};
pub use deployment::{
    CreateDeploymentRequest, Deployment, DeploymentEnv, DeploymentStatus,
};
pub use project::{
    CreateProjectRequest, Project, ProjectStatus, ProjectTemplate, UpdateProjectRequest,
};
pub use sandbox::{
    ChangeKind, CodeChange, Sandbox, SandboxExecRequest, SandboxExecResult, SandboxStatus,
};
pub use task::{AssignTaskRequest, CreateTaskRequest, Task, TaskClaim, TaskPriority, TaskStatus};
pub use world::{BiomeType, Chunk, VoxelBlock, VoxelKind, WorldSnapshot};

// ── Kanonische Typen (forge-core SYNC_CONTRACT v0.1) ─────────────────────────
pub use canonical::{AgentKind, FreeProvider};
pub use forge_plugin::{FfPluginCtx, FF_PLUGIN_ERROR, FF_PLUGIN_OK, abi as plugin_abi};
pub use tick::{DeterministicRng, RealmId, TickContext, WorldTick};

// ── SYNC_CONTRACT v0.1 §8 — Pflicht-Tests ────────────────────────────────────
#[cfg(test)]
mod sync_contract_tests {
    use super::*;
    use uuid::Uuid;

    // §8.2 — Event Equality: Gleicher Seed → gleiche RNG-Sequenz
    #[test]
    fn deterministic_rng_same_seed_same_sequence() {
        let mut rng_a = DeterministicRng::from_seed(0xDEAD_BEEF_CAFE_1234);
        let mut rng_b = DeterministicRng::from_seed(0xDEAD_BEEF_CAFE_1234);
        let seq_a: Vec<u64> = (0..50).map(|_| rng_a.next_u64()).collect();
        let seq_b: Vec<u64> = (0..50).map(|_| rng_b.next_u64()).collect();
        assert_eq!(seq_a, seq_b,
            "SYNC_CONTRACT §8.2: DeterministicRng muss mit gleichem Seed gleiche Folge liefern");
    }

    // §8.2 — TickContext: Gleiche Inputs → gleicher rng_seed
    #[test]
    fn tick_context_is_deterministic() {
        let realm = Uuid::new_v4();
        let ctx_a = TickContext::new(42, realm, 1);
        let ctx_b = TickContext::new(42, realm, 1);
        assert_eq!(ctx_a.rng_seed, ctx_b.rng_seed,
            "SYNC_CONTRACT §8.2: TickContext muss mit gleichen Inputs gleichen rng_seed liefern");
        assert_eq!(ctx_a, ctx_b);
    }

    // §8.2 — TickContext: Verschiedene Ticks → verschiedene Seeds
    #[test]
    fn different_ticks_produce_different_seeds() {
        let realm = Uuid::new_v4();
        let ctx_a = TickContext::new(1, realm, 0);
        let ctx_b = TickContext::new(2, realm, 0);
        assert_ne!(ctx_a.rng_seed, ctx_b.rng_seed,
            "SYNC_CONTRACT §8.2: Verschiedene Ticks müssen verschiedene Seeds erzeugen");
    }

    // §8.3 — Snapshot Round-Trip: Serialisierung/Deserialisierung
    #[test]
    fn world_snapshot_roundtrip() {
        let project_id = Uuid::new_v4();
        let snap = WorldSnapshot {
            id:          Uuid::new_v4(),
            project_id,
            chunks:      vec![],
            block_count: 42,
            message:     "SYNC_CONTRACT §8.3 test snapshot".into(),
            created_at:  chrono::Utc::now(),
        };
        let json = serde_json::to_string(&snap).unwrap();
        let restored: WorldSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap.block_count, restored.block_count,
            "SYNC_CONTRACT §8.3: Snapshot-Roundtrip muss block_count erhalten");
        assert_eq!(snap.message, restored.message);
        assert_eq!(snap.project_id, restored.project_id);
    }

    // §8.2 — FreeProvider Display: name() muss match-kompatibel sein
    #[test]
    fn free_provider_display_matches_name() {
        assert_eq!(FreeProvider::Groq.to_string(),      "groq");
        assert_eq!(FreeProvider::OpenRouter.to_string(), "openrouter");
        assert_eq!(FreeProvider::Ollama.to_string(),     "ollama");
        assert_eq!(FreeProvider::Cerebras.to_string(),   "cerebras");
        assert_eq!(FreeProvider::SambaNova.to_string(),  "sambanova");
        assert_eq!(FreeProvider::Mistral.to_string(),    "mistral");
        assert_eq!(FreeProvider::Gemini.to_string(),     "gemini");
    }
}
