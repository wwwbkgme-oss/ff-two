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
