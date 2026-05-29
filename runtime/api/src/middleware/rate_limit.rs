//! IP-basiertes Rate Limiting — Sliding-Window-Algorithmus.
//!
//! Konfiguration (ENV):
//! * `DEVSTUDIO_RATE_LIMIT_PER_MIN` — Anfragen pro IP pro Minute (default: 300)
//!
//! Antwort bei Überschreitung: `429 Too Many Requests`
//! mit Header `Retry-After: <sekunden>`.
//!
//! Implementierung: Axum `from_fn_with_state`-Middleware.
//! State: `Arc<RateLimiter>` — thread-sicher via `DashMap`.

use std::{
    collections::VecDeque,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde_json::json;
use tracing::debug;

// ── Konfiguration ─────────────────────────────────────────────────────────────

/// Sliding-Window-Größe.
const WINDOW: Duration = Duration::from_secs(60);

// ── RateLimiter ───────────────────────────────────────────────────────────────

/// Zentraler Rate-Limiter — kann als Axum-Extension geteilt werden.
pub struct RateLimiter {
    /// IP → Zeitstempel-Ringpuffer der Anfragen im aktuellen Fenster.
    buckets:   DashMap<IpAddr, VecDeque<Instant>>,
    /// Maximale Anfragen pro Fenster (pro IP).
    limit:     u32,
}

impl RateLimiter {
    /// Erstellt einen neuen Rate-Limiter.
    /// `limit` = max. Anfragen pro Minute pro IP.
    pub fn new(limit: u32) -> Arc<Self> {
        Arc::new(Self { buckets: DashMap::new(), limit })
    }

    /// Erstellt aus `DEVSTUDIO_RATE_LIMIT_PER_MIN` ENV-Variable (default: 300).
    pub fn from_env() -> Arc<Self> {
        let limit = std::env::var("DEVSTUDIO_RATE_LIMIT_PER_MIN")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(300);
        Self::new(limit)
    }

    /// Prüft und erhöht den Zähler für `ip`.
    /// Gibt `Ok(())` zurück wenn erlaubt, `Err(retry_after_secs)` wenn abgelehnt.
    pub fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let now = Instant::now();
        let mut bucket = self.buckets.entry(ip).or_default();

        // Veraltete Einträge außerhalb des Fensters entfernen
        bucket.retain(|t| now.duration_since(*t) < WINDOW);

        if bucket.len() as u32 >= self.limit {
            // Frühester Eintrag bestimmt Wartezeit
            let retry = bucket
                .front()
                .map(|t| WINDOW.saturating_sub(now.duration_since(*t)).as_secs() + 1)
                .unwrap_or(60);
            return Err(retry);
        }

        bucket.push_back(now);
        Ok(())
    }
}

// ── Axum-Middleware-Funktion ──────────────────────────────────────────────────

/// Axum-Middleware: prüft Rate-Limit vor jedem Request.
///
/// Füge sie dem Router mit:
/// ```rust
/// .layer(axum::middleware::from_fn_with_state(
///     limiter.clone(),
///     rate_limit_middleware,
/// ))
/// ```
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    connect_info:   Option<ConnectInfo<SocketAddr>>,
    request:        Request<Body>,
    next:           Next,
) -> Response {
    // IP aus ConnectInfo oder X-Forwarded-For extrahieren
    let ip = connect_info
        .map(|ci| ci.0.ip())
        .or_else(|| {
            request
                .headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
        })
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    match limiter.check(ip) {
        Ok(()) => {
            debug!(ip = %ip, "rate_limit: ok");
            next.run(request).await
        }
        Err(retry_after) => {
            debug!(ip = %ip, retry_after, "rate_limit: 429");
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                Json(json!({
                    "error": {
                        "code": "RATE_LIMITED",
                        "message": format!("Limit überschritten — bitte in {retry_after}s erneut versuchen")
                    }
                })),
            )
                .into_response()
        }
    }
}
