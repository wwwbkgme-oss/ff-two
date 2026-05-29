use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::{DeploymentEnv, DeploymentStatus};

/// Lebenszyklus-Events eines Deployments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DeploymentEvent {
    Started   { deployment_id: Uuid, project_id: Uuid, env: DeploymentEnv, version: String },
    Building  { deployment_id: Uuid },
    Testing   { deployment_id: Uuid },
    Scanning  { deployment_id: Uuid },
    Deploying { deployment_id: Uuid, target_url: String },
    Healthy   { deployment_id: Uuid, url: String },
    Failed    { deployment_id: Uuid, reason: String },
    RolledBack { deployment_id: Uuid },
    ApprovalAdded { deployment_id: Uuid, approver: String },
}
