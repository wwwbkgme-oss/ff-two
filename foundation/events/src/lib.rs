//! `foundation/events` — Event-Definitionen des Systems.
//!
//! ## BKG-Regel: Event-First
//! ```text
//! Events sind Wahrheit.
//! State ist Projektion.
//! ```
//!
//! Alle Zustandsänderungen im System werden als Events modelliert.
//! Domänen kommunizieren ausschließlich über Events — niemals über
//! direkte State-Manipulation.

pub mod agent;
pub mod deployment;
pub mod sandbox;
pub mod task;
pub mod world;

pub use agent::AgentEvent;
pub use deployment::DeploymentEvent;
pub use sandbox::SandboxEvent;
pub use task::TaskEvent;
pub use world::WorldEvent;
