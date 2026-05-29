use std::collections::HashMap;
use std::sync::Arc;

use types::AgentRole;
use crate::traits::DevRolePlugin;

/// Registriert alle verfügbaren Rollen-Plugins.
/// Wird von Orchestrator und runtime/server genutzt.
#[derive(Default)]
pub struct AgentRegistry {
    plugins: HashMap<AgentRole, Arc<dyn DevRolePlugin>>,
}

impl AgentRegistry {
    pub fn new() -> Self { Self::default() }

    /// Plugin für eine Rolle registrieren.
    pub fn register(&mut self, role: AgentRole, plugin: Arc<dyn DevRolePlugin>) {
        self.plugins.insert(role, plugin);
    }

    /// Plugin für eine Rolle abfragen.
    pub fn get(&self, role: &AgentRole) -> Option<&Arc<dyn DevRolePlugin>> {
        self.plugins.get(role)
    }

    /// Alle registrierten Rollen.
    pub fn roles(&self) -> impl Iterator<Item = &AgentRole> {
        self.plugins.keys()
    }
}

impl std::fmt::Debug for AgentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentRegistry")
            .field("roles", &self.plugins.keys().collect::<Vec<_>>())
            .finish()
    }
}
