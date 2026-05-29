//! RedisQueue — produktionsreife, persistente TaskQueue via Redis Lists.
//!
//! ## Aktivierung
//! `DEVSTUDIO_REDIS_URL=redis://localhost:6379` in der Umgebung setzen.
//! `runtime/server` wählt automatisch RedisQueue wenn die URL gesetzt ist.
//!
//! ## Implementierung (Reliable Queue Pattern)
//! - Queue-List:    `devstudio:queue:tasks`   (LPUSH / RPOP)
//! - In-Flight-Hash: `devstudio:queue:inflight` (HSET / HDEL)
//!
//! RPOP + HSET sind nicht atomar — bei Absturz zwischen beiden bleibt der Task
//! verloren. Für höhere Garantien: Redis Streams mit Consumer Groups (XREADGROUP).
//! Diese Implementierung ist für die meisten Anwendungsfälle ausreichend.
//!
//! ## Retry
//! `nack()` erhöht `attempts` und legt den Task zurück. Nach 5 Versuchen wird
//! der Task in die Dead-Letter-Liste `devstudio:queue:dead` verschoben.

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::Task;

use super::queue::{QueueItem, TaskQueue};

// ── Schlüssel ─────────────────────────────────────────────────────────────────

const KEY_QUEUE:    &str = "devstudio:queue:tasks";
const KEY_INFLIGHT: &str = "devstudio:queue:inflight";
const KEY_DEAD:     &str = "devstudio:queue:dead";
const MAX_ATTEMPTS: u32  = 5;

// ── RedisQueue ────────────────────────────────────────────────────────────────

/// Produktionsreife, persistente Queue via Redis Lists.
/// Überlebt Server-Neustarts; ermöglicht Multi-Instance-Betrieb.
pub struct RedisQueue {
    client: redis::Client,
}

impl RedisQueue {
    /// Erstellt eine RedisQueue. `url` z. B. `redis://127.0.0.1:6379`.
    pub fn new(url: &str) -> AppResult<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| AppError::Internal(format!("Redis-Client: {e}")))?;
        Ok(Self { client })
    }

    async fn conn(&self) -> AppResult<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Internal(format!("Redis-Verbindung: {e}")))
    }
}

impl std::fmt::Debug for RedisQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisQueue").finish()
    }
}

#[async_trait]
impl TaskQueue for RedisQueue {
    /// JSON-serialisierte Task in die Queue legen (LPUSH).
    async fn push(&self, task: Task) -> AppResult<()> {
        let json = serde_json::to_string(&QueueItem::new(task))
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut c = self.conn().await?;
        let _: () = c.lpush(KEY_QUEUE, json)
            .await
            .map_err(|e| AppError::Internal(format!("LPUSH: {e}")))?;
        Ok(())
    }

    /// Task aus der Queue holen (RPOP) und in In-Flight-Hash eintragen.
    async fn pop(&self) -> AppResult<Option<QueueItem>> {
        let mut c = self.conn().await?;
        let raw: Option<String> = c.rpop(KEY_QUEUE, None)
            .await
            .map_err(|e| AppError::Internal(format!("RPOP: {e}")))?;

        let Some(json) = raw else { return Ok(None) };

        let item: QueueItem = serde_json::from_str(&json)
            .map_err(|e| AppError::Internal(format!("queue deserialization: {e}")))?;

        // In-Flight verfolgen (Task-ID → JSON)
        let _: () = c.hset(KEY_INFLIGHT, item.task.id.to_string(), &json)
            .await
            .map_err(|e| AppError::Internal(format!("HSET inflight: {e}")))?;

        tracing::debug!(task_id = %item.task.id, "RedisQueue: pop");
        Ok(Some(item))
    }

    /// Task als erfolgreich abschließen — aus In-Flight entfernen.
    async fn ack(&self, id: Uuid) -> AppResult<()> {
        let mut c = self.conn().await?;
        let _: () = c.hdel(KEY_INFLIGHT, id.to_string())
            .await
            .map_err(|e| AppError::Internal(format!("HDEL inflight: {e}")))?;
        tracing::debug!(task_id = %id, "RedisQueue: ack");
        Ok(())
    }

    /// Task als fehlgeschlagen markieren — zurück in Queue oder Dead-Letter.
    async fn nack(&self, id: Uuid) -> AppResult<()> {
        let mut c = self.conn().await?;
        let raw: Option<String> = c.hget(KEY_INFLIGHT, id.to_string())
            .await
            .map_err(|e| AppError::Internal(format!("HGET inflight: {e}")))?;

        let Some(json) = raw else { return Ok(()); };

        let mut item: QueueItem = serde_json::from_str(&json)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // In-Flight-Eintrag entfernen
        let _: () = c.hdel(KEY_INFLIGHT, id.to_string())
            .await
            .map_err(|e| AppError::Internal(format!("HDEL nack: {e}")))?;

        item.attempts += 1;

        if item.attempts >= MAX_ATTEMPTS {
            // Dead-Letter-Queue
            let dead_json = serde_json::to_string(&item)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let _: () = c.lpush(KEY_DEAD, dead_json)
                .await
                .map_err(|e| AppError::Internal(format!("LPUSH dead: {e}")))?;
            tracing::warn!(task_id = %id, attempts = item.attempts, "RedisQueue: Task in Dead-Letter verschoben");
        } else {
            // Zurück in die Queue (am Ende für FIFO-Semantik)
            let retry_json = serde_json::to_string(&item)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let _: () = c.lpush(KEY_QUEUE, retry_json)
                .await
                .map_err(|e| AppError::Internal(format!("LPUSH retry: {e}")))?;
            tracing::debug!(task_id = %id, attempts = item.attempts, "RedisQueue: nack — re-queue");
        }
        Ok(())
    }

    /// Aktuelle Queue-Tiefe (LLEN).
    async fn depth(&self) -> AppResult<usize> {
        let mut c = self.conn().await?;
        let n: i64 = c.llen(KEY_QUEUE)
            .await
            .map_err(|e| AppError::Internal(format!("LLEN: {e}")))?;
        Ok(n.max(0) as usize)
    }
}
