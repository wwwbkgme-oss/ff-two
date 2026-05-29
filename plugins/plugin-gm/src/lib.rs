//! `forgefabrik.gm` — Game-Master Plugin (runtime-loadable cdylib).
//!
//! ## BKG-Architektur
//! * Ordner:     `plugins/plugin-gm/`
//! * Crate-Name: `gm`
//! * Plugin-ID:  `forgefabrik.gm`
//!
//! Steuert Spielregeln, Szenario-Events und narrative Abläufe.
//! Ermöglicht dem Host, Story-Elemente in die Dev-Simulation einzuschleusen.

use std::os::raw::c_char;

// ── Stabile C-ABI Metadaten ────────────────────────────────────────────────

static PLUGIN_ID:      &[u8] = b"forgefabrik.gm\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND:    &[u8] = b"runtime-plugin\0";

#[repr(C)]
pub struct PluginInfo {
    pub id:      *const c_char,
    pub version: *const c_char,
    pub kind:    *const c_char,
}

unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    static INFO: PluginInfo = PluginInfo {
        id:      PLUGIN_ID.as_ptr()      as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind:    PLUGIN_KIND.as_ptr()    as *const c_char,
    };
    &INFO
}

#[no_mangle]
pub extern "C" fn plugin_id() -> *const c_char {
    PLUGIN_ID.as_ptr() as *const c_char
}

// ── GM-Regel-Engine ───────────────────────────────────────────────────────

/// Bekannte GM-Event-Typen — numerischer Code für C-ABI-Kompatibilität.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GmEventKind {
    TaskCreated   = 0,
    TaskCompleted = 1,
    AgentBlocked  = 2,
    DeployFailed  = 3,
    SecurityAlert = 4,
    Milestone     = 5,
    Unknown       = 0xFF,
}

/// Bewertet eine Situation und gibt einen GM-Eingriffs-Score zurück.
///
/// * `blocked_agents`   — Anzahl blockierter Agenten
/// * `failed_deploys`   — Anzahl fehlgeschlagener Deployments
/// * `security_alerts`  — Anzahl offener Sicherheitshinweise
///
/// Rückgabe: 0 = kein Eingriff nötig, 100 = sofortiger Eingriff.
#[no_mangle]
pub extern "C" fn intervention_score(
    blocked_agents:  u32,
    failed_deploys:  u32,
    security_alerts: u32,
) -> u8 {
    let score = blocked_agents.saturating_mul(10)
        + failed_deploys.saturating_mul(20)
        + security_alerts.saturating_mul(30);
    score.min(100) as u8
}

/// Bestimmt die nächste empfohlene Aktion basierend auf dem System-Zustand.
///
/// Rückgabe-Codes:
/// * 0 = Abwarten
/// * 1 = Task neu zuweisen
/// * 2 = Sicherheits-Scan einleiten
/// * 3 = Deployment pausieren
/// * 4 = Rollback empfehlen
#[no_mangle]
pub extern "C" fn recommend_action(intervention: u8) -> u8 {
    match intervention {
        0..=19  => 0, // Abwarten
        20..=39 => 1, // Task neu zuweisen
        40..=59 => 2, // Security-Scan
        60..=79 => 3, // Deployment pausieren
        _       => 4, // Rollback
    }
}

/// Erzeugt einen deterministischen Szenario-Seed aus Projekt-ID und Tick-Nummer.
/// Nützlich für reproducible randomness in Tests und Replays.
#[no_mangle]
pub extern "C" fn scenario_seed(project_id_low: u64, tick: u64) -> u64 {
    // FNV-1a-inspirierter Hash — deterministisch und schnell
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in project_id_low.to_le_bytes().iter().chain(tick.to_le_bytes().iter()) {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
