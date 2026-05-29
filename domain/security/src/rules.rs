use crate::types::Severity;

/// Eine statische Scan-Regel.
pub struct Rule {
    pub id:       &'static str,
    pub severity: Severity,
    pub pattern:  &'static str,
    pub message:  &'static str,
}

/// Eingebaute Sicherheitsregeln (deterministisch, Replay-safe).
pub fn built_in() -> Vec<Rule> {
    vec![
        Rule { id: "SEC001", severity: Severity::Critical, pattern: "password",           message: "Mögliches hardcoded Passwort" },
        Rule { id: "SEC002", severity: Severity::Critical, pattern: "secret",             message: "Mögliches hardcoded Secret" },
        Rule { id: "SEC003", severity: Severity::High,     pattern: "api_key",            message: "Möglicher hardcoded API-Key" },
        Rule { id: "SEC004", severity: Severity::High,     pattern: "private_key",        message: "Möglicher hardcoded Private Key" },
        Rule { id: "SEC005", severity: Severity::High,     pattern: "SKIP_VERIFY",        message: "TLS-Verifikation deaktiviert" },
        Rule { id: "SEC006", severity: Severity::Medium,   pattern: "eval(",              message: "Code-Injection-Risiko via eval()" },
        Rule { id: "SEC007", severity: Severity::Medium,   pattern: "shell=True",         message: "Shell-Injection-Risiko (subprocess)" },
        Rule { id: "SEC008", severity: Severity::Medium,   pattern: "unsafe {",           message: "Unsafe-Rust-Block — bitte prüfen" },
        Rule { id: "SEC009", severity: Severity::Medium,   pattern: "TODO: fix security", message: "Aufgeschobene Sicherheitskorrektur" },
        Rule { id: "SEC010", severity: Severity::Low,      pattern: "console.log",        message: "Debug-Logging im Produktionscode" },
    ]
}
