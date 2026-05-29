//! `runtime/store` — Store-Trait + MemoryStore (Persistenzschicht).
//!
//! ## BKG-Regel
//! Runtime-Schicht: verwaltet Datenpersistenz (I/O).
//! Domänenlogik ist verboten — nur CRUD und Abfragen.

pub mod memory;
pub mod postgres;
pub mod store;

pub use memory::MemoryStore;
pub use postgres::PostgresStore;
pub use store::Store;
