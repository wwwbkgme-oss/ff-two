use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::Task;

use super::queue::{QueueItem, TaskQueue};

/// In-Memory-Queue auf Basis eines Tokio-mpsc-Channels.
/// Supports Work-Stealing durch mehrere parallele Consumer.
pub struct MemoryQueue {
    tx:        mpsc::Sender<QueueItem>,
    rx:        Mutex<mpsc::Receiver<QueueItem>>,
    in_flight: Mutex<HashMap<Uuid, QueueItem>>,
}

impl MemoryQueue {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx: Mutex::new(rx), in_flight: Mutex::new(HashMap::new()) }
    }
}

impl std::fmt::Debug for MemoryQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryQueue").finish()
    }
}

fn lock_err<T>(e: std::sync::PoisonError<T>) -> AppError {
    AppError::Internal(format!("mutex poisoned: {e}"))
}

#[async_trait]
impl TaskQueue for MemoryQueue {
    async fn push(&self, task: Task) -> AppResult<()> {
        self.tx.send(QueueItem::new(task)).await
            .map_err(|e| AppError::Internal(format!("queue send: {e}")))?;
        Ok(())
    }

    async fn pop(&self) -> AppResult<Option<QueueItem>> {
        let mut rx = self.rx.lock().map_err(lock_err)?;
        match rx.try_recv() {
            Ok(item) => {
                self.in_flight.lock().map_err(lock_err)?.insert(item.task.id, item.clone());
                Ok(Some(item))
            }
            Err(_) => Ok(None),
        }
    }

    async fn ack(&self, id: Uuid) -> AppResult<()> {
        self.in_flight.lock().map_err(lock_err)?.remove(&id);
        Ok(())
    }

    async fn nack(&self, id: Uuid) -> AppResult<()> {
        // Lock-Guard VOR dem await droppen (Send-Bound-Anforderung).
        let maybe = {
            let mut inf = self.in_flight.lock().map_err(lock_err)?;
            inf.remove(&id).map(|mut i| { i.attempts += 1; i })
        }; // guard dropped
        if let Some(item) = maybe {
            if item.attempts < 5 {
                self.tx.send(item).await
                    .map_err(|e| AppError::Internal(format!("queue send: {e}")))?;
            } else {
                tracing::warn!(task_id = %id, "task nach max. Versuchen verworfen");
            }
        }
        Ok(())
    }

    async fn depth(&self) -> AppResult<usize> {
        Ok(self.tx.max_capacity() - self.tx.capacity())
    }
}
