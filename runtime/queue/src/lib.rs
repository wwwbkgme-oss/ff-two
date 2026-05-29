//! `runtime/queue` — TaskQueue-Trait + MemoryQueue.
//!
//! ## BKG-Regel
//! Runtime-Schicht: verwaltet asynchrone Aufgabenverteilung.
//! Kein Domänenwissen — nur Dispatch-Mechanismus.

pub mod memory;
pub mod queue;

pub use memory::MemoryQueue;
pub use queue::{QueueItem, TaskQueue};
