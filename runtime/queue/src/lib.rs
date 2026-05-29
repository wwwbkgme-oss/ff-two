//! `runtime/queue` — TaskQueue-Trait + MemoryQueue + RedisQueue.
//!
//! ## BKG-Regel
//! Runtime-Schicht: verwaltet asynchrone Aufgabenverteilung.
//! Kein Domänenwissen — nur Dispatch-Mechanismus.
//!
//! ## Backend-Auswahl
//! - `MemoryQueue`  — Dev/Test (kein Redis nötig, verliert Daten bei Neustart)
//! - `RedisQueue`   — Production (persistiert, multi-instance-fähig)
//!   Aktivierung: `DEVSTUDIO_REDIS_URL=redis://...`

pub mod memory;
pub mod queue;
pub mod redis;

pub use memory::MemoryQueue;
pub use queue::{QueueItem, TaskQueue};
pub use redis::RedisQueue;
