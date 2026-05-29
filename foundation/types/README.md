# `foundation/types`

Single Source of Truth für alle Domain-Datentypen.

## BKG-Regeln

- Kein HTTP, keine DB, kein Netzwerk
- Keine Businesslogik, keine State-Mutation
- Nur Typdefinitionen, IDs, Datenstrukturen

## Module

| Modul | Inhalt |
|---|---|
| `agent` | `Agent`, `AgentRole` (6 Rollen), `AgentStatus` |
| `task` | `Task`, `TaskStatus`, `TaskPriority`, `TaskClaim`, Request-Typen |
| `project` | `Project`, `ProjectStatus`, `ProjectTemplate`, Request-Typen |
| `world` | `VoxelBlock`, `VoxelKind`, `Chunk`, `BiomeType`, `WorldSnapshot` |
| `sandbox` | `Sandbox`, `SandboxStatus`, `SandboxExecRequest`, `SandboxExecResult`, `CodeChange` |
| `deployment` | `Deployment`, `DeploymentStatus`, `DeploymentEnv`, Request-Typen |

## Verwendung

```rust
use types::{Project, Task, AgentRole, VoxelBlock};
```

Alle Typen implementieren `Debug`, `Clone`, `Serialize`, `Deserialize`.
