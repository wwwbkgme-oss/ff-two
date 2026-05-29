//! `runtime/config` — Settings-Loader (Infrastrukturschicht).
//!
//! ## BKG-Regel
//! Spricht mit der Außenwelt: liest Umgebungsvariablen, .env-Dateien.
//! Kein externer `config`-Crate — verhindert Namenskonflikt im Workspace.
//! Settings werden einmal geladen und als Arc<Settings> weitergegeben.

mod env_helpers;

pub mod settings;
pub use settings::{
    AgentSettings, AuthSettings, DatabaseSettings, DeploymentSettings,
    QueueSettings, SandboxSettings, SecuritySettings, ServerSettings,
    Settings, TracingSettings,
};
