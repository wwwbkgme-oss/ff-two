//! Token-Budget-Enforcement für `FreeLlmAgent`.
//!
//! Enthält die reine Berechnungslogik aus `plugins/plugin-economy`.
//! `FreeLlmAgent` ruft diese Funktionen vor jedem LLM-Call auf.
//!
//! ## Warum hier statt im Plugin?
//! `domain/agents` darf `plugins/` nicht importieren (Dependency-Richtung).
//! Die Budget-Logik ist deterministisch — sie gehört in die Domain-Schicht.
//! Das Plugin re-exportiert dieselben Formeln als C-ABI für externe Consumer.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, warn};

// ── Kosten-Berechnung ─────────────────────────────────────────────────────────

/// Modell-Tier beeinflusst Token-Kosten (Milli-Credits pro 1k Tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Small  = 0,  // 1 / 3
    Medium = 1,  // 3 / 15
    Large  = 2,  // 15 / 75
}

impl Default for ModelTier {
    fn default() -> Self { Self::Medium }
}

/// Schätzt Kosten in Milli-Credits (1 000 = 1 Credit).
pub fn estimate_cost_milli(tier: ModelTier, input_tokens: u64, output_tokens: u64) -> u64 {
    let (input_rate, output_rate) = match tier {
        ModelTier::Small  => (1u64,  3u64),
        ModelTier::Medium => (3u64,  15u64),
        ModelTier::Large  => (15u64, 75u64),
    };
    let input  = input_tokens.saturating_mul(input_rate)  / 1000;
    let output = output_tokens.saturating_mul(output_rate) / 1000;
    input.saturating_add(output)
}

/// Prüft ob genug Budget vorhanden ist.
pub fn budget_sufficient(remaining: u64, estimated: u64) -> bool {
    remaining >= estimated
}

/// Standard-Monatsbudget je Tier (Milli-Credits).
pub fn default_monthly_budget(tier: u8) -> u64 {
    match tier {
        0 => 10_000,   // Dev:      10 Credits
        1 => 100_000,  // Staging: 100 Credits
        _ => 500_000,  // Prod:    500 Credits
    }
}

// ── TokenBudget ───────────────────────────────────────────────────────────────

/// Thread-sicherer Token-Budget-Tracker für einen einzelnen Agenten.
///
/// Verwaltet das verbleibende Budget atomar — keine Mutex nötig.
#[derive(Debug)]
pub struct TokenBudget {
    remaining_milli: AtomicU64,
    model_tier:      ModelTier,
}

impl TokenBudget {
    /// Erstellt ein neues Budget.
    pub fn new(initial_milli: u64, tier: ModelTier) -> Arc<Self> {
        Arc::new(Self {
            remaining_milli: AtomicU64::new(initial_milli),
            model_tier:      tier,
        })
    }

    /// Standard-Budget für Entwicklung (10 Credits, Medium-Tier).
    pub fn dev() -> Arc<Self> {
        Self::new(default_monthly_budget(0), ModelTier::Medium)
    }

    /// Aktuell verbleibendes Budget in Milli-Credits.
    pub fn remaining(&self) -> u64 {
        self.remaining_milli.load(Ordering::Relaxed)
    }

    /// Prüft ob ein geplanter Aufruf (`max_tokens` Output-Tokens) finanzierbar ist.
    /// Input-Tokens werden geschätzt als `max_tokens / 4` (rough average).
    pub fn can_afford(&self, max_output_tokens: u32) -> bool {
        let estimated = estimate_cost_milli(
            self.model_tier,
            max_output_tokens as u64 / 4,  // geschätzte Input-Tokens
            max_output_tokens as u64,
        );
        let remaining = self.remaining();
        let ok = budget_sufficient(remaining, estimated);
        if !ok {
            warn!(
                remaining_milli = remaining,
                estimated_milli  = estimated,
                "TokenBudget: Budget erschöpft — Aufruf abgelehnt"
            );
        }
        ok
    }

    /// Verbucht tatsächlich verbrauchte Tokens nach einem erfolgreichen Aufruf.
    pub fn debit(&self, input_tokens: u32, output_tokens: u32) {
        let cost = estimate_cost_milli(
            self.model_tier,
            input_tokens  as u64,
            output_tokens as u64,
        );
        let prev = self.remaining_milli.fetch_sub(cost.min(self.remaining()), Ordering::Relaxed);
        debug!(
            cost_milli = cost,
            remaining  = prev.saturating_sub(cost),
            "TokenBudget: debitiert"
        );
    }
}
