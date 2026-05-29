use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use tracing_subscriber::EnvFilter;

use crate::env_helpers::*;

// ── Settings-Structs ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Settings {
    pub server:     ServerSettings,
    pub database:   DatabaseSettings,
    pub queue:      QueueSettings,
    pub sandbox:    SandboxSettings,
    pub security:   SecuritySettings,
    pub deployment: DeploymentSettings,
    pub agents:     AgentSettings,
    pub auth:       AuthSettings,
    pub tracing:    TracingSettings,
}

#[derive(Debug, Clone)]
pub struct ServerSettings { pub host: String, pub port: u16 }

impl ServerSettings {
    pub fn addr(&self) -> SocketAddr {
        let ip = IpAddr::from_str(&self.host).unwrap_or([0, 0, 0, 0].into());
        SocketAddr::new(ip, self.port)
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseSettings { pub url: String }

#[derive(Debug, Clone)]
pub struct QueueSettings { pub capacity: usize, pub redis_url: Option<String> }

#[derive(Debug, Clone)]
pub struct SandboxSettings {
    pub max_concurrent: usize,
    pub timeout_secs:   u64,
    pub work_dir:       String,
    pub use_docker:     bool,
}

#[derive(Debug, Clone)]
pub struct SecuritySettings {
    pub block_on_critical: bool,
    pub block_on_high:     bool,
}

#[derive(Debug, Clone)]
pub struct DeploymentSettings {
    pub require_consensus: usize,
    pub staging_url:       Option<String>,
    pub production_url:    Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentSettings {
    pub anthropic_api_key: Option<String>,
    pub openai_api_key:    Option<String>,
    pub model:             String,
    pub max_tokens:        u32,
    pub temperature:       f32,
}

#[derive(Debug, Clone)]
pub struct AuthSettings {
    pub jwt_secret:        String,
    pub token_expiry_secs: u64,
}

#[derive(Debug, Clone)]
pub struct TracingSettings { pub level: String, pub json: bool }

// ── Loader ────────────────────────────────────────────────────────────────

impl Settings {
    /// Lädt alle Settings aus `DEVSTUDIO_*` Umgebungsvariablen.
    /// Vorher wird `.env` eingelesen (falls vorhanden).
    pub fn from_env() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Self {
            server: ServerSettings {
                host: env_str("DEVSTUDIO_SERVER_HOST", "0.0.0.0"),
                port: env_u16("DEVSTUDIO_SERVER_PORT", 8080),
            },
            database: DatabaseSettings {
                url: env_str("DEVSTUDIO_DATABASE_URL", "sqlite::memory:"),
            },
            queue: QueueSettings {
                capacity:  env_usize("DEVSTUDIO_QUEUE_CAPACITY", 1024),
                redis_url: env_option("DEVSTUDIO_REDIS_URL"),
            },
            sandbox: SandboxSettings {
                max_concurrent: env_usize("DEVSTUDIO_SANDBOX_MAX",         10),
                timeout_secs:   env_u64("DEVSTUDIO_SANDBOX_TIMEOUT_SECS",  300),
                work_dir:       env_str("DEVSTUDIO_SANDBOX_WORK_DIR",      "/tmp/devstudio/sandboxes"),
                use_docker:     env_bool("DEVSTUDIO_SANDBOX_USE_DOCKER",   false),
            },
            security: SecuritySettings {
                block_on_critical: env_bool("DEVSTUDIO_SEC_BLOCK_CRITICAL", true),
                block_on_high:     env_bool("DEVSTUDIO_SEC_BLOCK_HIGH",     false),
            },
            deployment: DeploymentSettings {
                require_consensus: env_usize("DEVSTUDIO_DEPLOY_CONSENSUS",     3),
                staging_url:       env_option("DEVSTUDIO_DEPLOY_STAGING_URL"),
                production_url:    env_option("DEVSTUDIO_DEPLOY_PRODUCTION_URL"),
            },
            agents: AgentSettings {
                anthropic_api_key: env_option("ANTHROPIC_API_KEY"),
                openai_api_key:    env_option("OPENAI_API_KEY"),
                model:             env_str("DEVSTUDIO_AGENT_MODEL",       "claude-opus-4-5"),
                max_tokens:        env_u32("DEVSTUDIO_AGENT_MAX_TOKENS",  8192),
                temperature:       env_f32("DEVSTUDIO_AGENT_TEMPERATURE", 0.7),
            },
            auth: AuthSettings {
                jwt_secret:        env_str("DEVSTUDIO_JWT_SECRET",            "change-me-in-production"),
                token_expiry_secs: env_u64("DEVSTUDIO_JWT_EXPIRY_SECS",       86400),
            },
            tracing: TracingSettings {
                level: env_str("DEVSTUDIO_LOG_LEVEL", "info"),
                json:  env_bool("DEVSTUDIO_LOG_JSON",  false),
            },
        })
    }

    pub fn init_tracing(&self) {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&self.tracing.level));
        if self.tracing.json {
            tracing_subscriber::fmt().json().with_env_filter(filter).init();
        } else {
            tracing_subscriber::fmt().with_env_filter(filter).init();
        }
    }
}
