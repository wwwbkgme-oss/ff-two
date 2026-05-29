//! Task-Dispatch-Loop — pollt die Queue und ruft Agenten auf.
//!
//! Spawnt einen Tokio-Hintergrundtask der in einer Endlosschleife:
//!   1. Queue pollt (non-blocking `pop()`)
//!   2. Task auf `InProgress` setzt
//!   3. Passendes Plugin via `Orchestrator.select_plugin()` wählt
//!   4. `plugin.execute()` aufruft
//!   5. Ergebnis in Store persistiert + Task auf `Completed`/`Failed` setzt
//!   6. Queue `ack()` oder `nack()` aufruft
//!
//! ## Back-off
//! Bei leerer Queue: 500 ms warten.
//! Bei Fehlern: exponentielles Back-off bis max. 30 s.
//!
//! ## Sandbox (MVP)
//! Der erste Dispatch-Loop übergibt ein leeres `SandboxExecResult`.
//! Echte Sandbox-Ausführung kommt in einem späteren Sprint (→ NEXT.md P1).

use std::time::Duration;

use chrono::Utc;
use serde_json::json;
use tokio::{task::JoinHandle, time::sleep};
use tracing::{debug, error, info, warn};

use api::AppState;
use types::{SandboxExecResult, TaskStatus};

/// Intervall bei leerer Queue.
const IDLE_SLEEP: Duration = Duration::from_millis(500);
/// Start-Back-off bei Fehler.
const BACKOFF_START: Duration = Duration::from_millis(200);
/// Maximales Back-off bei wiederholten Fehlern.
const BACKOFF_MAX: Duration = Duration::from_secs(30);

// ── Öffentliche API ───────────────────────────────────────────────────────────

/// Spawnt den Dispatch-Loop als Hintergrundtask.
///
/// Der zurückgegebene `JoinHandle` kann ignoriert oder in `main()` abgewartet
/// werden. Der Loop läuft bis zum Prozessende.
pub fn spawn(state: AppState) -> JoinHandle<()> {
    tokio::spawn(run(state))
}

// ── Interne Implementierung ───────────────────────────────────────────────────

async fn run(state: AppState) {
    info!("dispatcher: gestartet");
    let mut backoff = BACKOFF_START;

    loop {
        match state.queue.pop().await {
            // ── Fehler beim Queue-Pop ──────────────────────────────────────────
            Err(e) => {
                error!(error = %e, backoff_ms = backoff.as_millis(), "dispatcher: queue.pop() fehlgeschlagen");
                sleep(backoff).await;
                backoff = (backoff * 2).min(BACKOFF_MAX);
            }

            // ── Queue leer ────────────────────────────────────────────────────
            Ok(None) => {
                backoff = IDLE_SLEEP;
                sleep(IDLE_SLEEP).await;
            }

            // ── Task vorhanden ────────────────────────────────────────────────
            Ok(Some(item)) => {
                backoff = BACKOFF_START; // Back-off zurücksetzen
                let task = item.task;
                debug!(task_id = %task.id, attempts = item.attempts, title = %task.title, "dispatcher: Task erhalten");

                // Task auf InProgress setzen
                {
                    let mut t = task.clone();
                    t.status     = TaskStatus::InProgress;
                    t.updated_at = Utc::now();
                    if let Err(e) = state.store.update_task(t).await {
                        warn!(task_id = %task.id, error = %e, "dispatcher: InProgress-Update fehlgeschlagen");
                    }
                }

                // Plugin wählen
                let Some(plugin) = state.orchestrator.select_plugin(&task).await else {
                    warn!(task_id = %task.id, required_role = ?task.required_role, "dispatcher: kein Plugin — nack");
                    set_failed(&state, &task, "kein passendes Plugin registriert").await;
                    let _ = state.queue.nack(task.id).await;
                    continue;
                };

                // MVP: leeres SandboxExecResult (echte Sandbox → NEXT.md P1)
                let exec = SandboxExecResult {
                    exit_code:   0,
                    stdout:      String::new(),
                    stderr:      String::new(),
                    duration_ms: 0,
                    changes:     vec![],
                };

                // Plugin ausführen
                match plugin.execute(&task, &exec).await {
                    Ok(output) => {
                        let mut t = task.clone();
                        t.status       = TaskStatus::Completed;
                        t.updated_at   = Utc::now();
                        t.completed_at = Some(Utc::now());
                        t.output       = Some(json!({
                            "summary":  output.summary,
                            "changes":  output.changes.len(),
                            "metadata": output.metadata,
                        }));

                        if let Err(e) = state.store.update_task(t).await {
                            error!(task_id = %task.id, error = %e, "dispatcher: Completed-Update fehlgeschlagen");
                        }
                        if let Err(e) = state.queue.ack(task.id).await {
                            error!(task_id = %task.id, error = %e, "dispatcher: ack fehlgeschlagen");
                        }
                        info!(
                            task_id = %task.id,
                            summary = %output.summary,
                            changes = output.changes.len(),
                            "dispatcher: Task abgeschlossen"
                        );
                    }
                    Err(e) => {
                        error!(task_id = %task.id, error = %e, "dispatcher: Plugin-Fehler — nack");
                        set_failed(&state, &task, &e.to_string()).await;
                        let _ = state.queue.nack(task.id).await;
                    }
                }
            }
        }
    }
}

/// Setzt einen Task auf `Failed` mit Fehlertext.
async fn set_failed(state: &AppState, task: &types::Task, reason: &str) {
    let mut t    = task.clone();
    t.status     = TaskStatus::Failed;
    t.error      = Some(reason.to_owned());
    t.updated_at = Utc::now();
    if let Err(e) = state.store.update_task(t).await {
        error!(task_id = %task.id, error = %e, "dispatcher: Failed-Update fehlgeschlagen");
    }
}
