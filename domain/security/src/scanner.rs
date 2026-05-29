use std::time::Instant;
use uuid::Uuid;
use ::types::CodeChange;
use crate::rules::{built_in, Rule};
use crate::types::{Finding, ScanResult, Severity};

/// Statischer Code-Analyser — deterministisch, kein Netzwerk.
pub struct Scanner {
    rules:             Vec<Rule>,
    block_on_critical: bool,
    block_on_high:     bool,
}

impl std::fmt::Debug for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scanner")
            .field("rule_count", &self.rules.len())
            .finish()
    }
}

impl Scanner {
    pub fn new(block_on_critical: bool, block_on_high: bool) -> Self {
        Self { rules: built_in(), block_on_critical, block_on_high }
    }

    /// Scannt eine Liste von Code-Änderungen. Deterministisch — gleicher
    /// Input liefert immer dasselbe Ergebnis (Replay-safe).
    pub fn scan(&self, changes: &[CodeChange]) -> ScanResult {
        let start    = Instant::now();
        let mut findings = Vec::new();

        for change in changes {
            for (ln, line) in change.content.lines().enumerate() {
                for rule in &self.rules {
                    if line.contains(rule.pattern) {
                        findings.push(Finding {
                            rule_id:  rule.id.into(),
                            severity: rule.severity.clone(),
                            message:  rule.message.into(),
                            path:     change.path.clone(),
                            line:     (ln + 1) as u32,
                            snippet:  line.trim().to_string(),
                        });
                    }
                }
            }
        }

        let block = (self.block_on_critical && findings.iter().any(|f| f.severity == Severity::Critical))
                 || (self.block_on_high      && findings.iter().any(|f| f.severity == Severity::High));

        let summary = if findings.is_empty() {
            "Keine Security-Issues gefunden".into()
        } else {
            format!("{} Findings: {} kritisch, {} hoch, {} mittel, {} niedrig",
                findings.len(),
                findings.iter().filter(|f| f.severity == Severity::Critical).count(),
                findings.iter().filter(|f| f.severity == Severity::High).count(),
                findings.iter().filter(|f| f.severity == Severity::Medium).count(),
                findings.iter().filter(|f| f.severity == Severity::Low).count())
        };

        ScanResult {
            scan_id:     Uuid::new_v4(),
            findings,
            passed:      !block,
            summary,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}
