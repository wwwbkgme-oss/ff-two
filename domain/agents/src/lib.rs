//! `domain/agents` — DevRolePlugin-Trait, 6 Rollen, Orchestrator.
//!
//! ## BKG-Regel
//! Reine Domänenlogik: kein HTTP, kein I/O, keine Store/Queue-Deps.
//! Der Orchestrator enthält Geschäftslogik (Aufgabenzuweisung, Konsens).
//! Der async Dispatch-Loop (Queue-Polling, Store-Persistenz) lebt in runtime/server.

pub mod budget;
pub mod llm_driver;
pub mod orchestrator;
pub mod registry;
pub mod roles;
pub mod traits;

pub use budget::{ModelTier, TokenBudget};
pub use llm_driver::{LlmDriver, LlmResponse, NullDriver};
pub use orchestrator::Orchestrator;
pub use registry::AgentRegistry;
pub use traits::{AgentOutput, BuildResult, DevRolePlugin, Review, WorldVisualizationPlugin};
