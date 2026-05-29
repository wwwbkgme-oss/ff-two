//! JWT Bearer-Token Authentifizierung — Middleware + Extractor + Token-Endpoint.
//!
//! ## Router-Level-Middleware (empfohlen)
//! ```rust,ignore
//! Router::new()
//!     .route("/projects", post(create))
//!     .layer(middleware::from_fn_with_state(state.clone(), require_auth))
//! ```
//!
//! ## Per-Handler-Extractor
//! ```rust,ignore
//! async fn handler(RequireAuth(claims): RequireAuth, ...) { }
//! ```
//!
//! ## Token erzeugen (Dev)
//! ```bash
//! curl -s -X POST http://localhost:8080/auth/token \
//!      -H 'Content-Type: application/json' \
//!      -d '{"sub":"dev","secret":"change-me-in-production-use-a-strong-random-value"}'
//! ```

use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;

// ── Claims ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

// ── Extractor ─────────────────────────────────────────────────────────────────

/// Handler-Extractor — erzwingt gültiges Bearer-Token.
pub struct RequireAuth(pub Claims);

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        verify_bearer(&parts.headers, state)
            .map(RequireAuth)
            .map_err(|m| unauthorized(&m))
    }
}

// ── Router-Level-Middleware ───────────────────────────────────────────────────

/// Axum-Middleware: prüft JWT für eine gesamte Router-Gruppe.
/// 401 wenn kein/ungültiges Token — leitet durch wenn ok.
pub async fn require_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next:    Next,
) -> Response {
    match verify_bearer(request.headers(), &state) {
        Ok(_claims) => next.run(request).await,
        Err(msg)    => unauthorized(&msg),
    }
}

// ── Token-Endpoint ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub sub:    String,
    pub secret: String,
}

/// `POST /auth/token` — Dev-Endpoint (nicht für Produktion).
pub async fn issue_token(
    State(state): State<AppState>,
    Json(body):   Json<TokenRequest>,
) -> Response {
    use jsonwebtoken::{encode, EncodingKey, Header};

    if body.secret != state.settings.auth.jwt_secret {
        return unauthorized("Falsches Secret");
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let claims = Claims {
        sub: body.sub,
        iat: now,
        exp: now + state.settings.auth.token_expiry_secs,
    };

    match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.settings.auth.jwt_secret.as_bytes()),
    ) {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({
                "token":      token,
                "expires_in": state.settings.auth.token_expiry_secs,
            })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "code": "TOKEN_ERROR", "message": e.to_string() } })),
        ).into_response(),
    }
}

// ── Interne Hilfen ────────────────────────────────────────────────────────────

fn verify_bearer(headers: &axum::http::HeaderMap, state: &AppState) -> Result<Claims, String> {
    let header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or("Authorization header fehlt")?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or("Bearer-Token erwartet")?;

    let key        = DecodingKey::from_secret(state.settings.auth.jwt_secret.as_bytes());
    let validation = Validation::default();

    decode::<Claims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => "Token abgelaufen".to_owned(),
            ErrorKind::InvalidSignature => "Ungültige Signatur".to_owned(),
            ErrorKind::InvalidToken     => "Ungültiges Token-Format".to_owned(),
            _                           => format!("Token ungültig: {e}"),
        })
}

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": { "code": "UNAUTHORIZED", "message": msg } })),
    ).into_response()
}
