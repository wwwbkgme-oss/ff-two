//! `foundation/types` — Single Source of Truth für alle Domain-Datentypen.
//!
//! ## BKG-Regeln
//! - Kein HTTP, keine DB, kein Netzwerk
//! - Keine Businesslogik, keine State-Mutation
//! - Nur Typdefinitionen, IDs, Datenstrukturen

pub mod agent;
pub mod deployment;
pub mod project;
pub mod sandbox;
pub mod task;
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
