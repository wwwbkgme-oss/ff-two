use std::sync::Arc;

use tracing::info;
use types::{AgentRole, Task, TaskClaim};

use crate::registry::AgentRegistry;
use crate::traits::DevRolePlugin;

/// Reine Domänen-Orchestrierungslogik.
///
/// Enthält:
/// * Aufgaben-Zuweisung (welcher Agent nimmt welche Task?)
/// * Konsens-Prüfung
/// * Plug-in-Auswahl nach Rolle
///
/// **KEIN** I/O, **KEIN** Queue-Polling — das lebt in `runtime/server`.
#[derive(Debug)]
pub struct Orchestrator {
    registry: Arc<AgentRegistry>,
    /// Mindestanzahl Konsens-Votes für eine Aufgabe.
    pub consensus_threshold: usize,
}

impl Orchestrator {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry, consensus_threshold: 3 }
    }

    /// Bestimmt, welches Plugin eine Task übernehmen soll.
    /// Gibt `None` zurück, wenn kein passendes Plugin registriert ist.
    pub async fn select_plugin(&self, task: &Task) -> Option<Arc<dyn DevRolePlugin>> {
        if let Some(role) = &task.required_role {
            if let Some(plugin) = self.registry.get(role) {
                if plugin.claim_task(task).await.is_some() {
                    return Some(Arc::clone(plugin));
                }
            }
        }
        // Fallback: erstes Plugin, das die Task claimen möchte
        for role in self.registry.roles() {
            if let Some(plugin) = self.registry.get(role) {
                if plugin.claim_task(task).await.is_some() {
                    return Some(Arc::clone(plugin));
                }
            }
        }
        None
    }

    /// Prüft, ob genügend Konsens-Votes für eine Task vorliegen.
    pub fn consensus_met(&self, task: &Task) -> bool {
        task.has_consensus(self.consensus_threshold)
    }
}
