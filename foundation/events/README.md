# `foundation/events`

Event-Definitionen des Systems — Single Source of Truth für alle Domain-Events.

## BKG-Regeln

- Kein HTTP, keine DB, kein Netzwerk
- Events sind reine Datenstrukturen (keine Handler, keine Side-Effects)
- Jedes Event beschreibt eine abgeschlossene Zustandsänderung

## Module

| Modul | Events |
|---|---|
| `agent` | `AgentRegistered`, `AgentTaskClaimed`, `AgentTaskCompleted` |
| `task` | `TaskCreated`, `TaskAssigned`, `TaskStatusChanged`, `TaskCompleted` |
| `world` | `BlockPlaced`, `BlockRemoved`, `FileVisualized`, `SnapshotCreated` |
| `sandbox` | `SandboxCreated`, `SandboxExecuted`, `SandboxDestroyed` |
| `deployment` | `DeploymentStarted`, `DeploymentStageChanged`, `DeploymentCompleted` |

## Verwendung

```rust
use events::WorldEvent;

// WorldState sendet Events über einen Broadcast-Channel:
let rx = world_state.subscribe();
```

Die `WorldEvent`-Varianten werden von `runtime/api` als SSE-Stream an Clients gesendet.
