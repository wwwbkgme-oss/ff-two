//! `domain/security` — Statische Code-Analyse, Regeln, Findings.
//!
//! ## BKG-Regel
//! Reine Domänenlogik: kein HTTP, keine DB, kein Netzwerk.
//! Deterministisch: gleiche Eingaben → gleiche Findings (Replay-safe).

pub mod rules;
pub mod scanner;
pub mod types;

pub use scanner::Scanner;
pub use types::{Finding, ScanRequest, ScanResult, Severity};
