//! `devstudio-types` — the Single Source of Truth for all domain models.
//!
//! Every other crate imports types from here; no other crate may re-define
//! the same concept.  This crate has zero DevStudio-internal dependencies.

pub mod agent;
pub mod deployment;
pub mod project;
pub mod sandbox;
pub mod task;
pub mod world;

// Flat re-exports for ergonomic use: `use devstudio_types::Project;`
pub use agent::{Agent, AgentRole, AgentStatus};
pub use deployment::{Deployment, DeploymentEnv, DeploymentStatus, CreateDeploymentRequest};
pub use project::{Project, ProjectStatus, ProjectTemplate, CreateProjectRequest, UpdateProjectRequest};
pub use sandbox::{Sandbox, SandboxStatus, CodeChange, ChangeKind, SandboxExecRequest, SandboxExecResult};
pub use task::{Task, TaskClaim, TaskPriority, TaskStatus, CreateTaskRequest, AssignTaskRequest};
pub use world::{BiomeType, Chunk, VoxelBlock, VoxelKind, WorldEvent, WorldSnapshot};
