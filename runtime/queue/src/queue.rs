use async_trait::async_trait;
use uuid::Uuid;

use errors::AppResult;
use types::Task;

/// Ein Element in der Dispatch-Queue.
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub task:     Task,
    pub attempts: u32,
}
impl QueueItem { pub fn new(task: Task) -> Self { Self { task, attempts: 0 } } }

/// Abstraktes Queue-Interface — Implementierungen: Memory, Redis, …
#[async_trait]
pub trait TaskQueue: Send + Sync + 'static {
    /// Task in die Queue einreihen.
    async fn push(&self, task: Task)    -> AppResult<()>;
    /// Höchstpriorisierte Task aus der Queue holen (non-blocking).
    async fn pop(&self)                 -> AppResult<Option<QueueItem>>;
    /// Erfolgreiche Verarbeitung bestätigen.
    async fn ack(&self, id: Uuid)       -> AppResult<()>;
    /// Verarbeitung fehlgeschlagen — Task zurück in die Queue.
    async fn nack(&self, id: Uuid)      -> AppResult<()>;
    /// Aktuelle Queue-Tiefe.
    async fn depth(&self)               -> AppResult<usize>;
}
