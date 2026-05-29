//! `runtime/sandbox` — SandboxManager-Trait + Local/Docker-Implementierung.
//!
//! ## BKG-Regel
//! Runtime-Schicht: verwaltet isolierte Ausführungsumgebungen (I/O: Prozesse, Dateisystem).
//! Domänenlogik ist verboten — nur Sandbox-Lifecycle und Code-Ausführung.

pub mod local;
pub mod manager;

pub use local::LocalSandboxManager;
pub use manager::SandboxManager;
