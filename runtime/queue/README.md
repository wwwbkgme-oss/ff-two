# `runtime/queue`

`TaskQueue`-Trait und `MemoryQueue` — abstraktes Queue-Interface für den Task-Dispatch-Loop.

## TaskQueue-Trait

```rust
#[async_trait]
pub trait TaskQueue: Send + Sync + 'static {
    async fn push(&self, task: Task)   -> AppResult<()>;
    async fn pop(&self)                -> AppResult<Option<QueueItem>>;
    async fn ack(&self, id: Uuid)      -> AppResult<()>;   // Erfolg bestätigen
    async fn nack(&self, id: Uuid)     -> AppResult<()>;   // Zurück in Queue (Retry)
    async fn depth(&self)              -> AppResult<usize>; // Aktuelle Tiefe
}
```

## QueueItem

```rust
pub struct QueueItem {
    pub task:     Task,
    pub attempts: u32,  // Erhöht bei jedem nack()
}
```

## MemoryQueue

In-Memory-Queue auf Basis eines `tokio::sync::mpsc`-Channels mit in-flight-Tracking.

```rust
use queue::MemoryQueue;

let queue: Arc<dyn TaskQueue> = Arc::new(MemoryQueue::new(1024));
```

### Retry-Verhalten

- `nack()` legt den `QueueItem` mit `attempts += 1` zurück in den Channel
- Nach **5 fehlgeschlagenen Versuchen** wird die Task verworfen (`warn!`-Log)
- Lock-Guards werden vor `await`-Points gedroppt (`Send`-Anforderung für Tokio)

### Einschränkungen

- Rein in-process: Daten gehen beim Neustart verloren
- Kein Multi-Consumer-Support über Prozessgrenzen hinweg
- Kapazität fix bei Initialisierung (`capacity`-Parameter)

## Geplante Backends

- `RedisQueue` — persistente Queue via Redis Streams (Mehrprozess-fähig, Replay-fähig)

Siehe `NEXT.md` für Priorisierung.
