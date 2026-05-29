//! Prometheus-Metriken — `GET /metrics` Endpunkt.
//!
//! ## Metriken
//! * `devstudio_requests_total{method, path, status}` — HTTP-Request-Counter
//! * `devstudio_request_duration_seconds{method, path}` — Latenz-Histogramm
//! * `devstudio_queue_depth` — aktuelle Queue-Tiefe (Gauge)
//! * `devstudio_tasks_completed_total` — abgeschlossene Tasks
//! * `devstudio_tasks_failed_total` — fehlgeschlagene Tasks
//! * `devstudio_llm_calls_total{provider}` — LLM-Provider-Aufrufe
//! * `devstudio_llm_errors_total{provider}` — fehlgeschlagene LLM-Aufrufe
//!
//! ## Verwendung
//! ```bash
//! curl http://localhost:8080/metrics
//! # → Prometheus text format
//! ```

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::OnceLock;

static PROMETHEUS: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialisiert den Prometheus-Exporter — einmalig beim Server-Start aufrufen.
pub fn init_prometheus() -> PrometheusHandle {
    PROMETHEUS.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("Prometheus-Recorder konnte nicht installiert werden")
    })
    .clone()
}

/// Handler für `GET /metrics` — gibt den aktuellen Prometheus-Scrape-Output zurück.
pub async fn metrics_handler() -> impl IntoResponse {
    match PROMETHEUS.get() {
        Some(handle) => (
            StatusCode::OK,
            [("Content-Type", "text/plain; version=0.0.4; charset=utf-8")],
            handle.render(),
        ).into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Prometheus nicht initialisiert",
        ).into_response(),
    }
}

// ── Metriken-Hilfsmakros ──────────────────────────────────────────────────────

/// HTTP-Request gezählt. Aufgerufen am Ende jedes Requests.
pub fn record_request(method: &str, path: &str, status: u16) {
    metrics::counter!("devstudio_requests_total",
        "method" => method.to_owned(),
        "path"   => path.to_owned(),
        "status" => status.to_string(),
    ).increment(1);
}

/// Task-Abschluss registrieren.
pub fn record_task_completed() {
    metrics::counter!("devstudio_tasks_completed_total").increment(1);
}

/// Task-Fehler registrieren.
pub fn record_task_failed() {
    metrics::counter!("devstudio_tasks_failed_total").increment(1);
}

/// LLM-Provider-Aufruf registrieren.
pub fn record_llm_call(provider: &str) {
    metrics::counter!("devstudio_llm_calls_total",
        "provider" => provider.to_owned(),
    ).increment(1);
}

/// LLM-Fehler registrieren.
pub fn record_llm_error(provider: &str) {
    metrics::counter!("devstudio_llm_errors_total",
        "provider" => provider.to_owned(),
    ).increment(1);
}

/// Queue-Tiefe setzen.
pub fn set_queue_depth(depth: f64) {
    metrics::gauge!("devstudio_queue_depth").set(depth);
}
