use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The technology template used to seed a new project world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectTemplate {
    WebApi, CliTool, Library, Microservice, FullStack, DataPipeline, Custom,
}

impl std::fmt::Display for ProjectTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_value(self).unwrap().as_str().unwrap_or("custom"))
    }
}

/// Lifecycle state of a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus { Initializing, Active, Paused, Archived, Failed }

/// A project world — the top-level unit of development in DevStudio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id:          Uuid,
    pub name:        String,
    pub description: String,
    pub template:    ProjectTemplate,
    pub status:      ProjectStatus,
    /// Deterministic seed for the voxel world layout.
    pub world_seed:  String,
    pub branch:      String,
    pub metadata:    serde_json::Value,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
}

impl Project {
    pub fn new(
        name:        impl Into<String>,
        description: impl Into<String>,
        template:    ProjectTemplate,
    ) -> Self {
        let now = Utc::now();
        Self {
            id:          Uuid::new_v4(),
            name:        name.into(),
            description: description.into(),
            template,
            status:      ProjectStatus::Initializing,
            world_seed:  format!("{:#010x}", rand::random::<u32>()),
            branch:      "main".into(),
            metadata:    serde_json::Value::Object(Default::default()),
            created_at:  now,
            updated_at:  now,
        }
    }
}

/// Request body — create project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name:        String,
    pub description: Option<String>,
    pub template:    Option<ProjectTemplate>,
}

/// Request body — update project metadata or status.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub description: Option<String>,
    pub status:      Option<ProjectStatus>,
    pub branch:      Option<String>,
    pub metadata:    Option<serde_json::Value>,
}
