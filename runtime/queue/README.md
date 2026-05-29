# `runtime/queue`

`TaskQueue`-Trait + `MemoryQueue` + `RedisQueue` — abstraktes Queue-Interface.

## Backend-Auswahl

| Backend | Aktivierung | Eigenschaften |
|---|---|---|
| `MemoryQueue` | Standard (kein ENV nötig) | In-process, verliert Daten bei Neustart |
| `RedisQueue` | `DEVSTUDIO_REDIS_URL=redis://...` | Persistiert, multi-instance-fähig |

## TaskQueue-Trait

```rust
#[async_trait]
pub trait TaskQueue: Send + Sync + 'static {
    async fn push(&self, task: Task)   -> AppResult<()>;
    async fn pop(&self)                -> AppResult<Option<QueueItem>>;
    async fn ack(&self, id: Uuid)      -> AppResult<()>;   // Erfolg
    async fn nack(&self, id: Uuid)     -> AppResult<()>;   // Retry / Dead-Letter
    async fn depth(&self)              -> AppResult<usize>;
}
```

## MemoryQueue

In-Memory-Queue auf Basis eines `tokio::sync::mpsc`-Channels mit in-flight-Tracking.

```rust
let queue: Arc<dyn TaskQueue> = Arc::new(MemoryQueue::new(1024));
```

Retry: `nack()` legt Task nach max. 5 Versuchen in Dead-Letter. Lock-Guards werden vor `await`-Points gedroppt.

## RedisQueue

Produktionsreife Queue via Redis Lists — überlebt Server-Neustarts.

```rust
let queue: Arc<dyn TaskQueue> = Arc::new(RedisQueue::new("redis://localhost:6379")?);
```

**Implementierung (Reliable Queue Pattern):**
- Queue-List: `devstudio:queue:tasks` (LPUSH / RPOP)
- In-Flight-Hash: `devstudio:queue:inflight` (HSET / HDEL)
- Dead-Letter-List: `devstudio:queue:dead` (nach 5 fehlgeschlagenen Versuchen)

**Auto-Select in `runtime/server`:**
```bash
DEVSTUDIO_REDIS_URL=redis://localhost:6379 cargo run --bin devstudio
# Startup-Log: "Queue: RedisQueue"
```
