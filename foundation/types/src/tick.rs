//! Deterministische Simulations-Primitive — kanonische Typen (forge-core SYNC_CONTRACT v0.1).
//!
//! **Pflicht:** Domain-Code MUSS `TickContext` nutzen, niemals `Utc::now()` oder `thread_rng()`.
//!
//! Identisch mit `forge-core/foundation/types/src/tick.rs`.

use serde::{Deserialize, Serialize};

/// Absoluter Simulations-Schrittzähler. Monoton steigend, kein Überlauf.
/// Kanonischer Name: `WorldTick` (forge-core SYNC_CONTRACT §3).
pub type WorldTick = u64;

/// Opaker Realm-Identifier (World-Shard).
pub type RealmId = uuid::Uuid;

/// Gesamter Kontext den eine Domain-Funktion für Determinismus benötigt.
///
/// Wird vom Runtime konstruiert; Domain-Code liest nur.
/// Ersetzt `Utc::now()` und `thread_rng()` in allen Domain-Crates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TickContext {
    /// Absoluter Simulationsschritt.
    pub tick:        WorldTick,
    /// World-Shard.
    pub realm:       RealmId,
    /// Per-Tick-Seed — deterministisch aus tick + realm abgeleitet.
    pub rng_seed:    u64,
    /// Ticks seit letztem Kontext.
    pub delta_ticks: u64,
}

impl TickContext {
    /// Standardkonstruktion. `rng_seed = blake3(tick_bytes ++ realm_bytes)[..8]`.
    pub fn new(tick: WorldTick, realm: RealmId, delta_ticks: u64) -> Self {
        let mut h = blake3::Hasher::new();
        h.update(&tick.to_le_bytes());
        h.update(realm.as_bytes());
        let digest = h.finalize();
        let rng_seed = u64::from_le_bytes(
            digest.as_bytes()[..8].try_into().unwrap_or([0u8; 8])
        );
        Self { tick, realm, rng_seed, delta_ticks }
    }
}

/// Deterministischer, seedierbarer PRNG — nutze statt `thread_rng()` in Domain-Code.
///
/// Algorithmus: xorshift64 (schnell, deterministisch, keine globale Zustand).
#[derive(Debug, Clone)]
pub struct DeterministicRng(u64);

impl DeterministicRng {
    pub fn from_seed(seed: u64) -> Self { Self(seed.max(1)) }
    pub fn from_context(ctx: &TickContext) -> Self { Self::from_seed(ctx.rng_seed) }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x; x
    }
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
    pub fn next_range(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next_u64() % n }
    }
}
