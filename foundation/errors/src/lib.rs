//! `foundation/errors` — Fehlerverträge des Systems.
//!
//! ## BKG-Regel
//! Dieses Crate kennt **keine** Infrastruktur.
//! Kein HTTP-Status, kein Axum, kein SQLx.
//!
//! Die HTTP-Antwort-Konvertierung (`IntoResponse`) lebt in `runtime/api`.
//! DB-Fehler werden von `runtime/store` nach `AppError::Internal` gemappt.

use thiserror::Error;
use uuid::Uuid;

/// Zentraler Anwendungsfehler — alle Schichten geben diesen Typ zurück.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("sandbox error: {0}")]
    Sandbox(String),

    #[error("agent error: {0}")]
    Agent(String),

    #[error("security scan failed: {0}")]
    SecurityScan(String),

    #[error("deployment error: {0}")]
    Deployment(String),

    /// Fangbecken für unerwartete Fehler — Ursache wird als String eingebettet,
    /// damit keine Infra-Typen aus dem Foundation-Layer ragen.
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Hilfsmethode: Ressource nicht gefunden.
    pub fn not_found(kind: &str, id: Uuid) -> Self {
        Self::NotFound(format!("{kind} {id} not found"))
    }

    /// Hilfsmethode: Beliebigen `std::error::Error` in `Internal` wrappen.
    pub fn internal(e: impl std::error::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

/// Kurzalias — alle Handler und Services geben `AppResult<T>` zurück.
pub type AppResult<T> = std::result::Result<T, AppError>;
