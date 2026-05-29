//! `forgefabrik.agents` — AI-Agenten Plugin (runtime-loadable cdylib).
//!
//! ## BKG-Architektur
//! * Ordner:      `plugins/plugin-agents/`
//! * Crate-Name:  `agents-runtime`  (domain/agents belegt "agents")
//! * Plugin-ID:   `forgefabrik.agents`
//!
//! Dieses Crate ist eine eigenständige Dynamic Library.
//! Enthält Plugin-seitige Logik für Aufgabenvergabe und Konsens-Voting.

use std::os::raw::c_char;

// ── Stabile C-ABI Metadaten ────────────────────────────────────────────────

static PLUGIN_ID:      &[u8] = b"forgefabrik.agents\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND:    &[u8] = b"runtime-plugin\0";

/// Stabile Plugin-Metadaten — `#[repr(C)]` garantiert ABI-Kompatibilität.
#[repr(C)]
pub struct PluginInfo {
    pub id:      *const c_char,
    pub version: *const c_char,
    pub kind:    *const c_char,
}

unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

/// Gibt statische Metadaten dieses Plugins zurück.
#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    static INFO: PluginInfo = PluginInfo {
        id:      PLUGIN_ID.as_ptr()      as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind:    PLUGIN_KIND.as_ptr()    as *const c_char,
    };
    &INFO
}

/// Null-terminierter Plugin-ID-String.
#[no_mangle]
pub extern "C" fn plugin_id() -> *const c_char {
    PLUGIN_ID.as_ptr() as *const c_char
}

// ── Agenten-Rollen ────────────────────────────────────────────────────────

/// Bekannte Agenten-Rollen — spiegelt `types::AgentRole` als C-kompatibles Enum.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRoleId {
    Requirements = 0,
    Architecture = 1,
    Coding       = 2,
    Testing      = 3,
    Security     = 4,
    Deployment   = 5,
    Unknown      = 0xFF,
}

impl AgentRoleId {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Requirements,
            1 => Self::Architecture,
            2 => Self::Coding,
            3 => Self::Testing,
            4 => Self::Security,
            5 => Self::Deployment,
            _ => Self::Unknown,
        }
    }
}

/// Prüft, ob ein Agent mit der gegebenen Rolle eine Task übernehmen kann.
///
/// * `required_role_id` — Gewünschte Rolle (255 = beliebig)
/// * `agent_role_id`    — Tatsächliche Rolle des Agenten
///
/// Rückgabe: 1 = kann übernehmen, 0 = kann nicht.
#[no_mangle]
pub extern "C" fn can_claim(required_role_id: u8, agent_role_id: u8) -> u8 {
    let required = AgentRoleId::from_u8(required_role_id);
    let agent    = AgentRoleId::from_u8(agent_role_id);
    // 0xFF = keine Rolle gefordert → jeder Agent kann übernehmen
    if required_role_id == 0xFF || required == agent { 1 } else { 0 }
}

/// Prüft, ob ausreichend Konsens-Votes für eine Aufgabe vorliegen.
///
/// * `votes_count`     — Anzahl der abgegebenen Stimmen
/// * `threshold`       — Mindestanzahl
///
/// Rückgabe: 1 = Konsens erreicht, 0 = noch nicht.
#[no_mangle]
pub extern "C" fn consensus_met(votes_count: u32, threshold: u32) -> u8 {
    if votes_count >= threshold { 1 } else { 0 }
}

/// Berechnet eine einfache Prioritäts-Gewichtung für die Task-Zuweisung.
/// Niedrigerer Rückgabewert = höhere Priorität.
///
/// * `attempts` — Bisherige Versuche (erhöht Dringlichkeit)
/// * `age_secs` — Alter der Task in Sekunden
#[no_mangle]
pub extern "C" fn task_priority_score(attempts: u32, age_secs: u64) -> u64 {
    // Ältere Tasks + häufige Versuche werden dringlicher
    let attempt_weight = (attempts as u64).saturating_mul(60);
    age_secs.saturating_add(attempt_weight)
}

// ── SYNC_CONTRACT v0.1 §6 — Kanonische Plugin-Lifecycle-Hooks ────────────────

struct AgentsPluginLifecycle;
impl AgentsPluginLifecycle {
    fn new() -> Self { Self }
    fn init(&mut self, _ctx: &types::forge_plugin::FfPluginCtx) -> Result<(), String> {
        tracing::debug!("forgefabrik.agents: ff_plugin_init"); Ok(())
    }
    fn tick(&mut self, _tick: u64) -> Result<(), String> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), String> {
        tracing::debug!("forgefabrik.agents: ff_plugin_shutdown"); Ok(())
    }
}
types::export_forgefabrik_plugin!(AgentsPluginLifecycle, AgentsPluginLifecycle::new());
