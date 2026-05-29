# `runtime/queue`

`TaskQueue`-Trait und `MemoryQueue` — abstraktes Queue-Interface.

## TaskQueue-Trait

```rust
#[async_trait]
pub trait TaskQueue: Send + Sync + 'static {
    async fn push(&self, task: Task)   -> AppResult<()>;
    async fn pop(&self)                -> AppResult<Option<QueueItem>>;
    async fn ack(&self, id: Uuid)      -> AppResult<()>;  // Erfolg bestätigen
    async fn nack(&self, id: Uuid)     -> AppResult<()>;  // Zurück in Queue
    async fn depth(&self)              -> AppResult<usize>;
}
```

## MemoryQueue

In-Memory-Queue auf Basis eines `tokio::sync::mpsc`-Channels.

```rust
use queue::MemoryQueue;

let queue = Arc::new(MemoryQueue::new(1024)); // Kapazität: 1024 Elemente
```

### Retry-Verhalten

- `nack()` legt die Task mit erhöhtem `attempts`-Counter zurück
- Nach 5 fehlgeschlagenen Versuchen wird die Task verworfen (geloggt als `warn`)
- Lock-Guards werden vor `await`-Points gedroppt (Rust `Send`-Anforderung)

## Geplante Backends

- `RedisQueue` — persistente Queue für Mehrprozess-Setups
