# `domain/deployment`

Deployment-Pipeline-Manager und Konsenslogik.

## BKG-Regeln

- Reine Domänenlogik: kein HTTP, keine DB
- Der Manager beschreibt Pipeline-Stufen (Build → Test → Scan → Deploy)
- Tatsächliche I/O-Aktionen (Container starten, DB schreiben) liegen in `runtime`

## Pipeline-Stufen

```
Build → Test → SecurityScan → Deploy → HealthCheck
```

Jede Stufe aktualisiert `Deployment.status` und schreibt einen Log-Eintrag.

## API

```rust
use deployment::{DeploymentManager, PipelineConfig};

let manager = DeploymentManager::new(PipelineConfig {
    require_consensus: 3,
    staging_url:       Some("https://staging.example.com".into()),
    production_url:    Some("https://example.com".into()),
});

// Pipeline ausführen
let deployment = manager.run(deployment).await?;

// Rollback
let deployment = manager.rollback(deployment)?;

// Konsens prüfen
let ok = manager.consensus_met(&approvals);
```

## Deployment-Environments

| Env | URL |
|---|---|
| `Staging` | `staging_url` aus Config (Fallback: `https://staging.devstudio.local/{project_id}`) |
| `Production` | `production_url` aus Config (Fallback: `https://devstudio.local/{project_id}`) |
| `Preview` | `https://preview-{deployment_id}.devstudio.local` |
