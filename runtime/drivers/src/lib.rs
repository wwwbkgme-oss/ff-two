//! `runtime/drivers` — Infrastruktur-Adapter für ForgeFabrik DevStudio.
//!
//! ## BKG / ARCHITECTURE.md
//! Dieses Crate lebt in der **runtime-Schicht** und implementiert Traits
//! aus der **domain-Schicht** (`domain/agents::LlmDriver`).
//!
//! Niemals von `domain/` oder `plugins/` importiert werden.
//! Verdrahtung ausschließlich in `runtime/server/main.rs`.
//!
//! ## Aktuelle Driver
//! * `llm::FreeProviderDriver` — Free LLM über 8 kostenlose Provider

pub mod llm;

pub use llm::FreeProviderDriver;
