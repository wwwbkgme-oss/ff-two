//! `runtime/sandbox` — SandboxManager-Trait + Local/Docker-Implementierungen.
//!
//! ## BKG-Regel
//! Runtime-Schicht: verwaltet isolierte Ausführungsumgebungen (I/O: Prozesse, Docker).
//! Domänenlogik ist verboten — nur Sandbox-Lifecycle und Code-Ausführung.
//!
//! ## Backend-Auswahl
//! - `LocalSandboxManager`  — Dev/Test (kein Docker nötig)
//! - `DockerSandboxManager` — Production (Container-Isolation)
//!   Aktivierung: `DEVSTUDIO_SANDBOX_USE_DOCKER=true`

pub mod docker;
pub mod local;
pub mod manager;

pub use docker::DockerSandboxManager;
pub use local::LocalSandboxManager;
pub use manager::SandboxManager;
