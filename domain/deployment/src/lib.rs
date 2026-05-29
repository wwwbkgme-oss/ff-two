//! `domain/deployment` — Deployment-Pipeline, Konsenslogik.
//!
//! ## BKG-Regel
//! Reine Domänenlogik: kein HTTP, keine DB.
//! Der Manager beschreibt die Pipeline-Stufen (build→test→scan→deploy).
//! Tatsächliche I/O-Aktionen (Container starten, DB schreiben) liegen in runtime.

pub mod manager;
pub mod pipeline;

pub use manager::DeploymentManager;
pub use pipeline::{PipelineConfig, PipelineStage};
