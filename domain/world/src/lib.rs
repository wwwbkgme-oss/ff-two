//! `domain/world` — Voxel-Welt, WorldState, Koordinaten-Mapping.
//!
//! ## BKG-Regel
//! Reine Fachdomäne: kein HTTP, keine DB, kein Netzwerk.
//! Kommuniziert Änderungen ausschließlich via `WorldEvent`.

pub mod state;
pub mod visualizer;

pub use state::WorldState;
