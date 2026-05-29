# `foundation/events`

Event-Definitionen des Systems — Single Source of Truth für alle Domain-Events.

## BKG-Regeln

- Kein HTTP, keine DB, kein Netzwerk
- Events sind unveränderliche Datenstrukturen (keine Handler, keine Side-Effects)
- Jedes Event beschreibt eine **abgeschlossene** Zustandsänderung im Vergangen­heitstempus
- Domänen kommunizieren ausschließlich über Events — niemals über direkte State-Mutation

## Enum-Übersicht

### `AgentEvent`

```rust
AgentEvent::Spawned       { agent_id, role, model }
AgentEvent::TaskClaimed   { agent_id, task_id }
AgentEvent::TaskStarted   { agent_id, task_id }
AgentEvent::TaskDone      { agent_id, task_id, success }
AgentEvent::ReviewDone    { agent_id, task_id, approved }
AgentEvent::StatusChanged { agent_id, new_status }
AgentEvent::Offline       { agent_id }
```

### `TaskEvent`

```rust
TaskEvent::Created       { task_id, project_id, title, priority }
TaskEvent::Queued        { task_id }
TaskEvent::Claimed       { task_id, agent_id, role }
TaskEvent::Started       { task_id, sandbox_id }
TaskEvent::Completed     { task_id, output }
TaskEvent::Failed        { task_id, reason }
TaskEvent::Cancelled     { task_id }
TaskEvent::VoteAdded     { task_id, voter, approved }
TaskEvent::ConsensusMet  { task_id, threshold }
```

### `WorldEvent`

```rust
WorldEvent::BlockPlaced    { block: VoxelBlock }
WorldEvent::BlockRemoved   { x, y, z }
WorldEvent::ChunkLoaded    { cx, cy, cz }
WorldEvent::SnapshotCreated { snapshot: WorldSnapshot }
WorldEvent::FileVisualized  { path, block_count }
```

Diese Varianten werden von `runtime/api` via SSE an Clients gestreamt (`GET /projects/{id}/world/stream`).

### `DeploymentEvent`

```rust
DeploymentEvent::Started       { deployment_id, project_id, env, version }
DeploymentEvent::Building      { deployment_id }
DeploymentEvent::Testing       { deployment_id }
DeploymentEvent::Scanning      { deployment_id }
DeploymentEvent::Deploying     { deployment_id, target_url }
DeploymentEvent::Healthy       { deployment_id, url }
DeploymentEvent::Failed        { deployment_id, reason }
DeploymentEvent::RolledBack    { deployment_id }
DeploymentEvent::ApprovalAdded { deployment_id, approver }
```

### `SandboxEvent`

```rust
SandboxEvent::Created   { sandbox_id, project_id, work_dir }
SandboxEvent::Ready     { sandbox_id }
SandboxEvent::Executing { sandbox_id, command }
SandboxEvent::Snapshot  { sandbox_id, snapshot_id }
SandboxEvent::Restored  { sandbox_id, snapshot_id }
SandboxEvent::Destroyed { sandbox_id }
SandboxEvent::Error     { sandbox_id, reason }
```

## Verwendung

```rust
use events::{AgentEvent, DeploymentEvent, SandboxEvent, TaskEvent, WorldEvent};

// WorldState sendet Events über einen Broadcast-Channel:
let rx = world_state.subscribe(); // broadcast::Receiver<WorldEvent>

// In einem Task-Handler:
while let Ok(event) = rx.recv().await {
    match event {
        WorldEvent::BlockPlaced { block } => { /* ... */ }
        WorldEvent::SnapshotCreated { snapshot } => { /* ... */ }
        _ => {}
    }
}
```

## Serialisierung

Alle Enums nutzen `#[serde(tag = "type", rename_all = "kebab-case")]`.

```json
{ "type": "block-placed", "block": { "x": 10, "y": 0, "z": 5, ... } }
{ "type": "task-completed", "task_id": "...", "output": { ... } }
```
