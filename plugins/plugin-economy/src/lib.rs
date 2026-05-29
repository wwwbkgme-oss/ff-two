//! `forgefabrik.economy` — Ressourcen-Ökonomie Plugin (runtime-loadable cdylib).
//!
//! ## BKG-Architektur
//! * Ordner:     `plugins/plugin-economy/`
//! * Crate-Name: `economy`
//! * Plugin-ID:  `forgefabrik.economy`
//!
//! Verwaltet Token-Budgets, Rate-Limiting und Kosten-Tracking für KI-Agenten.
//! Enthält keine externen I/O-Abhängigkeiten — reine Berechnungslogik.

use std::os::raw::c_char;

// ── Stabile C-ABI Metadaten ────────────────────────────────────────────────

static PLUGIN_ID:      &[u8] = b"forgefabrik.economy\0";
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

// ── Token-Budget ───────────────────────────────────────────────────────────

/// Modell-Kosten-Tier — bestimmt Token-Preis in Milli-Credits (1/1000 Credit).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Kleine Modelle (z. B. Claude Haiku) — günstig.
    Small    = 0,
    /// Mittlere Modelle (z. B. Claude Sonnet) — Standard.
    Medium   = 1,
    /// Große Modelle (z. B. Claude Opus) — teuer.
    Large    = 2,
    Unknown  = 0xFF,
}

/// Berechnet die geschätzten Token-Kosten in Milli-Credits.
///
/// * `tier`        — Modell-Tier (0=Small, 1=Medium, 2=Large)
/// * `input_toks`  — Anzahl Input-Tokens
/// * `output_toks` — Anzahl Output-Tokens
///
/// Rückgabe: Kosten in Milli-Credits (1000 = 1 Credit).
#[no_mangle]
pub extern "C" fn estimate_cost_milli(tier: u8, input_toks: u64, output_toks: u64) -> u64 {
    // Preise pro 1000 Tokens in Milli-Credits (Beispielwerte)
    let (input_rate, output_rate) = match tier {
        0 => (1u64,  3u64),   // Small:  0.001 / 0.003 Credits pro 1k
        1 => (3u64, 15u64),   // Medium: 0.003 / 0.015
        _ => (15u64, 75u64),  // Large:  0.015 / 0.075
    };
    let input_cost  = input_toks.saturating_mul(input_rate)  / 1000;
    let output_cost = output_toks.saturating_mul(output_rate) / 1000;
    input_cost.saturating_add(output_cost)
}

/// Prüft, ob genügend Budget für einen geplanten API-Aufruf vorhanden ist.
///
/// * `budget_remaining` — Verbleibendes Budget in Milli-Credits
/// * `estimated_cost`   — Geschätzte Kosten in Milli-Credits
///
/// Rückgabe: 1 = Budget reicht aus, 0 = Budget erschöpft.
#[no_mangle]
pub extern "C" fn budget_sufficient(budget_remaining: u64, estimated_cost: u64) -> u8 {
    if budget_remaining >= estimated_cost { 1 } else { 0 }
}

/// Berechnet den Throttle-Delay in Millisekunden basierend auf der Rate-Limit-Policy.
///
/// * `calls_last_minute` — Anzahl Aufrufe in der letzten Minute
/// * `rate_limit`        — Erlaubte Aufrufe pro Minute
///
/// Rückgabe: Wartezeit in ms (0 = kein Throttling nötig).
#[no_mangle]
pub extern "C" fn throttle_delay_ms(calls_last_minute: u32, rate_limit: u32) -> u64 {
    if calls_last_minute < rate_limit { return 0; }
    // Exponentieller Back-off: je weiter über dem Limit, desto länger warten
    let overflow = calls_last_minute.saturating_sub(rate_limit) as u64;
    let base_ms  = 60_000u64 / rate_limit.max(1) as u64;
    base_ms.saturating_mul(1u64 << overflow.min(10))
}

/// Gibt das monatliche Standard-Budget für einen Agent-Tier zurück (in Milli-Credits).
///
/// * `agent_tier` — 0=Dev, 1=Staging, 2=Production
#[no_mangle]
pub extern "C" fn default_monthly_budget(agent_tier: u8) -> u64 {
    match agent_tier {
        0 => 10_000,   // Dev:        10 Credits
        1 => 100_000,  // Staging:   100 Credits
        _ => 500_000,  // Prod:      500 Credits
    }
}

// ── SYNC_CONTRACT v0.1 §6 — Kanonische Plugin-Lifecycle-Hooks ────────────────

struct EconomyPluginLifecycle;
impl EconomyPluginLifecycle {
    fn new() -> Self { Self }
    fn init(&mut self, _ctx: &types::forge_plugin::FfPluginCtx) -> Result<(), String> {
        tracing::debug!("forgefabrik.economy: ff_plugin_init"); Ok(())
    }
    fn tick(&mut self, _tick: u64) -> Result<(), String> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), String> {
        tracing::debug!("forgefabrik.economy: ff_plugin_shutdown"); Ok(())
    }
}
types::export_forgefabrik_plugin!(EconomyPluginLifecycle, EconomyPluginLifecycle::new());
