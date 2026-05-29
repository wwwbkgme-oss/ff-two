use serde::{Deserialize, Serialize};

/// Konfiguration der Deployment-Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub require_consensus:  usize,
    pub staging_url:        Option<String>,
    pub production_url:     Option<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self { require_consensus: 3, staging_url: None, production_url: None }
    }
}

/// Eine Stufe der Deployment-Pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PipelineStage {
    Build, Test, SecurityScan, Deploy, HealthCheck,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Build        => "build",
            Self::Test         => "test",
            Self::SecurityScan => "security-scan",
            Self::Deploy       => "deploy",
            Self::HealthCheck  => "health-check",
        };
        write!(f, "{s}")
    }
}
