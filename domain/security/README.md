# `domain/security`

Statische Code-Analyse — deterministischer Scanner für Security-Findings.

## BKG-Regeln

- Reine Domänenlogik: kein HTTP, keine DB, kein Netzwerk
- Deterministisch: gleiche Eingaben → gleiche Findings (Replay-safe)
- Regeln sind reine Pattern-Matches auf Zeilenbasis

## Änderungen

`ScanRequest` hat ein optionales `project_id`-Feld — wenn gesetzt, wird das Ergebnis
über `AppState.scan_reviews` gespeichert und ist via `GET /projects/{id}/reviews` abrufbar.

## API

```rust
use security::{Scanner, ScanResult};

let scanner = Scanner::new(
    block_on_critical: true,  // Deployment blockieren bei Critical
    block_on_high: false,
);

let result: ScanResult = scanner.scan(&code_changes);

// result.passed   — false wenn blockierende Findings vorliegen
// result.findings — Liste aller gefundenen Issues
// result.summary  — Lesbare Zusammenfassung
```

## Finding-Schweregrade

| Schweregrad | Bedeutung |
|---|---|
| `Critical` | Sofortiger Handlungsbedarf (Credentials, RCE) |
| `High` | Hohe Dringlichkeit (unsichere Konfiguration) |
| `Medium` | Mittlere Dringlichkeit |
| `Low` | Hinweise, Best-Practice-Abweichungen |

## Eingebaute Regeln

Eingebaute Regeln erkennen u. a.:
- Hartcodierte Passwörter und Secrets
- `SKIP_VERIFY` / TLS-Deaktivierung
- `eval()` in dynamischen Sprachen
- Bekannte unsichere Muster

Eigene Regeln können über die `Rule`-Struktur hinzugefügt werden.
