use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ::types::CodeChange;

/// Schweregrad eines Security-Findings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Low, Medium, High, Critical }

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Info => "INFO", Self::Low => "LOW", Self::Medium => "MEDIUM",
                     Self::High => "HIGH", Self::Critical => "CRITICAL" }
    }
}

/// Ein einzelnes Security-Finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id:  String,
    pub severity: Severity,
    pub message:  String,
    pub path:     String,
    pub line:     u32,
    pub snippet:  String,
}

/// Ergebnis eines Scan-Laufs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id:     Uuid,
    pub findings:    Vec<Finding>,
    pub passed:      bool,
    pub summary:     String,
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn critical_count(&self) -> usize { self.findings.iter().filter(|f| f.severity == Severity::Critical).count() }
    pub fn high_count(&self)     -> usize { self.findings.iter().filter(|f| f.severity == Severity::High).count() }
}

/// API-Request für einen Ad-hoc-Scan.
#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub changes: Vec<CodeChange>,
    pub context: Option<String>,
}
