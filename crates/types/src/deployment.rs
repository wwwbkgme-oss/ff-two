use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentEnv { Staging, Production, Preview }

impl std::fmt::Display for DeploymentEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Staging    => "staging",
            Self::Production => "production",
            Self::Preview    => "preview",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending, Building, Testing, SecurityScan,
    Deploying, Healthy, Degraded, Failed, RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id:          Uuid,
    pub project_id:  Uuid,
    pub env:         DeploymentEnv,
    pub status:      DeploymentStatus,
    pub version:     String,
    pub commit_sha:  Option<String>,
    /// Agent approvals collected before the deploy was authorised.
    pub approvals:   Vec<String>,
    pub logs:        Vec<String>,
    pub url:         Option<String>,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
    pub deployed_at: Option<DateTime<Utc>>,
}

impl Deployment {
    pub fn new(project_id: Uuid, env: DeploymentEnv, version: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(), project_id, env, status: DeploymentStatus::Pending,
            version: version.into(), commit_sha: None, approvals: Vec::new(),
            logs: Vec::new(), url: None, created_at: now, updated_at: now, deployed_at: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub env:        DeploymentEnv,
    pub version:    Option<String>,
    pub commit_sha: Option<String>,
}
