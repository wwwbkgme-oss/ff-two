//! JWT Bearer-Token Authentifizierung als Axum-Extractor.
//!
//! ## Verwendung
//!
//! ```rust
//! use api::middleware::auth::RequireAuth;
//!
//! // Handler-Parameter — erzwingt gültiges JWT
//! async fn my_handler(
//!     RequireAuth(claims): RequireAuth,
//!     State(s): State<AppState>,
//! ) -> ApiResult<Json<Value>> {
//!     // claims.sub enthält den Aufrufer
//!     Ok(Json(json!({ "user": claims.sub })))
//! }
//! ```
//!
//! ## Token erzeugen (CLI-Beispiel)
//! ```bash
//! # Temporärer Dev-Token (Ablauf: 24 h)
//! curl -s -X POST http://localhost:8080/auth/token \
//!      -H 'Content-Type: application/json' \
//!      -d '{"sub":"dev","secret":"change-me-in-production-use-a-strong-random-value"}'
//! ```
//!
//! Das JWT-Secret wird aus `DEVSTUDIO_JWT_SECRET` gelesen.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;

// ── Claims ────────────────────────────────────────────────────────────────────

/// JWT-Payload — wird in jedem authentifizierten Request verfügbar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — Aufrufer-Identität (z. B. User-ID oder API-Key-ID).
    pub sub: String,
    /// Expiry — Unix-Timestamp (Sekunden).
    pub exp: u64,
    /// Issued-At — Unix-Timestamp (Sekunden).
    pub iat: u64,
}

// ── Extractor ─────────────────────────────────────────────────────────────────

/// Axum-Extractor: erzwingt gültiges Bearer-Token und gibt Claims zurück.
///
/// Antwortet mit 401 wenn:
/// * `Authorization`-Header fehlt
/// * Token kein Bearer-Token
/// * Signatur ungültig
/// * Token abgelaufen
pub struct RequireAuth(pub Claims);

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts:  &mut Parts,
        state:  &AppState,
    ) -> Result<Self, Self::Rejection> {
        // ── Authorization-Header lesen ──────────────────────────────────────
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| unauthorized("Authorization header fehlt"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| unauthorized("Bearer-Token erwartet"))?;

        // ── JWT validieren ─────────────────────────────────────────────────
        let secret  = state.settings.auth.jwt_secret.as_bytes();
        let key     = DecodingKey::from_secret(secret);
        let validation = Validation::default();

        let data = decode::<Claims>(token, &key, &validation).map_err(|e| {
            let msg = match e.kind() {
                ErrorKind::ExpiredSignature => "Token abgelaufen",
                ErrorKind::InvalidSignature => "Ungültige Signatur",
                ErrorKind::InvalidToken     => "Ungültiges Token-Format",
                _                           => "Token-Validierung fehlgeschlagen",
            };
            unauthorized(msg)
        })?;

        Ok(RequireAuth(data.claims))
    }
}

// ── Token-Endpoint ────────────────────────────────────────────────────────────

/// Request-Body für `POST /auth/token`.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub sub:    String,
    pub secret: String,
}

/// Erzeugt einen JWT (nur für Development/Tests — kein Produktions-Endpoint).
///
/// In Produktion: externe Auth-Provider (Keycloak, Auth0, etc.) verwenden.
pub async fn issue_token(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<TokenRequest>,
) -> Response {
    use jsonwebtoken::{encode, EncodingKey, Header};

    // Secret prüfen (verhindert versehentliche Token-Ausstellung ohne Konfiguration)
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

    let key = EncodingKey::from_secret(state.settings.auth.jwt_secret.as_bytes());
    match encode(&Header::default(), &claims, &key) {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({ "token": token, "expires_in": state.settings.auth.token_expiry_secs })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": { "code": "TOKEN_ERROR", "message": e.to_string() } })),
        )
            .into_response(),
    }
}

// ── Hilfsfunktion ─────────────────────────────────────────────────────────────

fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": { "code": "UNAUTHORIZED", "message": msg } })),
    )
        .into_response()
}
