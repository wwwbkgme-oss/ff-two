//! `foundation/events` — Event-Definitionen des Systems.
//!
//! ## BKG-Regel: Event-First
//! ```text
//! Events sind Wahrheit.
//! State ist Projektion.
//! ```
//!
//! Alle Zustandsänderungen im System werden als Events modelliert.
//! Domänen kommunizieren ausschließlich über Events — niemals über
//! direkte State-Manipulation.
//!
//! ## forge-core SYNC_CONTRACT v0.1
//! Enthält §8.1 Pflicht-Test: deterministischer Replay.

pub mod agent;
pub mod deployment;
pub mod sandbox;
pub mod task;
pub mod world;

pub use agent::AgentEvent;
pub use deployment::DeploymentEvent;
pub use sandbox::SandboxEvent;
pub use task::TaskEvent;
pub use world::WorldEvent;

// ── SYNC_CONTRACT v0.1 §8.1 — Deterministischer Replay-Test (Pflicht) ─────────
#[cfg(test)]
mod sync_contract_replay_tests {
    use super::WorldEvent;

    /// Minimaler State-Projektor — reines Fold über Events.
    /// Demonstriert das Single-Mutation-Path-Muster (§5).
    #[derive(Default, Debug, Clone, PartialEq)]
    struct WorldBlockCount(usize);

    fn apply_event(state: &WorldBlockCount, event: &WorldEvent) -> WorldBlockCount {
        match event {
            WorldEvent::BlockPlaced { .. }               => WorldBlockCount(state.0 + 1),
            WorldEvent::BlockRemoved { .. }              => WorldBlockCount(state.0.saturating_sub(1)),
            WorldEvent::FileVisualized { block_count, .. } => WorldBlockCount(state.0 + block_count),
            _                                             => WorldBlockCount(state.0),
        }
    }

    fn replay(events: &[WorldEvent]) -> WorldBlockCount {
        events.iter().fold(WorldBlockCount::default(), |s, e| apply_event(&s, e))
    }

    /// §8.1 — Deterministischer Replay-Test.
    ///
    /// Gleiche Events → gleicher State-Hash (hier: block_count als Proxy).
    /// `assert_eq!(replay(events), snapshot.state_hash)` — Pflicht.
    #[test]
    fn replay_produces_deterministic_state() {
        let events = vec![
            WorldEvent::FileVisualized { path: "src/main.rs".into(), block_count: 10 },
            WorldEvent::FileVisualized { path: "src/lib.rs".into(),  block_count: 5  },
            WorldEvent::BlockRemoved   { x: 0, y: 0, z: 0 },
        ];

        let state_a = replay(&events);
        let state_b = replay(&events);

        // §8.1: Replay muss identischen State-Hash liefern.
        assert_eq!(state_a, state_b,
            "SYNC_CONTRACT §8.1: replay(events) muss deterministisch sein");
        assert_eq!(state_a.0, 14,
            "SYNC_CONTRACT §8.1: 10 + 5 - 1 = 14 Blöcke erwartet");
    }

    /// §8.1 — Replay ist reihenfolge-sensitiv (kein Kommutativitäts-Annahmen).
    #[test]
    fn replay_order_matters() {
        let events_forward = vec![
            WorldEvent::FileVisualized { path: "a".into(), block_count: 5 },
            WorldEvent::BlockRemoved   { x: 0, y: 0, z: 0 },
        ];
        let events_reversed = vec![
            WorldEvent::BlockRemoved   { x: 0, y: 0, z: 0 },
            WorldEvent::FileVisualized { path: "a".into(), block_count: 5 },
        ];

        // Beide geben 4 — aber Reihenfolge muss respektiert werden.
        // BlockRemoved auf leerem State = saturating_sub(0) = 0, dann +5 = 5.
        let state_forward  = replay(&events_forward);
        let state_reversed = replay(&events_reversed);
        assert_eq!(state_forward.0,  4, "forward:  5 - 1 = 4");
        assert_eq!(state_reversed.0, 5, "reversed: 0 (saturate) + 5 = 5");
        // Reihenfolge ändert das Ergebnis — wie erwartet.
        assert_ne!(state_forward, state_reversed,
            "SYNC_CONTRACT §8.1: Event-Reihenfolge ist semantisch relevant");
    }

    /// §8.2 — Event-Gleichheits-Test via JSON-Serialisierung.
    #[test]
    fn events_with_same_data_produce_equal_json() {
        let e_a = WorldEvent::FileVisualized { path: "main.rs".into(), block_count: 7 };
        let e_b = WorldEvent::FileVisualized { path: "main.rs".into(), block_count: 7 };
        let json_a = serde_json::to_string(&e_a).unwrap();
        let json_b = serde_json::to_string(&e_b).unwrap();
        assert_eq!(json_a, json_b,
            "SYNC_CONTRACT §8.2: gleiche Event-Daten → gleiche JSON-Darstellung");
    }
}
