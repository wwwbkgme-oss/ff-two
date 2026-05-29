//! `runtime/api` — Axum HTTP-Router, Handler, Middleware, AppState.
//!
//! ## BKG-Regel
//! Runtime-Schicht: spricht mit der Außenwelt (HTTP).
//! Enthält die IntoResponse-Impl für AppError (HTTP-Status-Mapping).
//! Keine Fachlogik — delegiert an domain-Crates.

pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;

pub use errors::{ApiError, ApiResult};
pub use middleware::auth::{Claims, RequireAuth};
pub use state::AppState;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;

/// Für Integration-Tests — baut den Router ohne zu binden.
pub fn build_test_app(state: AppState) -> Router {
    router::build(state)
}

/// Bindet und startet den HTTP-Server.
pub async fn serve(state: AppState) -> Result<()> {
    let addr     = state.settings.server.addr();
    let router   = router::build(state);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, "HTTP server listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async { signal::ctrl_c().await.expect("ctrl-c handler") };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("sigterm handler").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutdown signal received");
}
