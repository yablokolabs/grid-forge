use grid_forge_domain::Environment;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: Environment,
    pub bind_addr: SocketAddr,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub object_storage: ObjectStorageConfig,
    pub feature_flags: FeatureFlags,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub demo_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub require_citations: bool,
    pub enable_mock_llm: bool,
    pub enable_connector_writes: bool,
    pub require_human_approval_for_customer_drafts: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let environment = match env_var("GRID_FORGE_ENV", "development").as_str() {
            "production" => Environment::Production,
            "demo" => Environment::Demo,
            "development" | "dev" => Environment::Development,
            other => return Err(format!("unsupported GRID_FORGE_ENV={other}")),
        };

        Ok(Self {
            environment,
            bind_addr: env_var("GRID_FORGE_BIND_ADDR", "0.0.0.0:8080")
                .parse()
                .map_err(|err| format!("invalid GRID_FORGE_BIND_ADDR: {err}"))?,
            database: DatabaseConfig {
                url: env_var(
                    "GRID_FORGE_DATABASE_URL",
                    "postgres://grid_forge:grid_forge@localhost:5432/grid_forge",
                ),
                max_connections: env_var("GRID_FORGE_DATABASE_MAX_CONNECTIONS", "5")
                    .parse()
                    .map_err(|err| format!("invalid GRID_FORGE_DATABASE_MAX_CONNECTIONS: {err}"))?,
            },
            auth: AuthConfig {
                jwt_secret: env_var("GRID_FORGE_JWT_SECRET", "development-only-change-me"),
                demo_mode: env_bool("GRID_FORGE_DEMO_MODE", true),
            },
            object_storage: ObjectStorageConfig {
                endpoint: env_var("GRID_FORGE_OBJECT_STORE_ENDPOINT", "http://localhost:9000"),
                bucket: env_var("GRID_FORGE_OBJECT_STORE_BUCKET", "grid-forge-documents"),
                access_key: env_var("GRID_FORGE_OBJECT_STORE_ACCESS_KEY", "minioadmin"),
                secret_key: env_var("GRID_FORGE_OBJECT_STORE_SECRET_KEY", "minioadmin"),
            },
            feature_flags: FeatureFlags {
                require_citations: env_bool("GRID_FORGE_REQUIRE_CITATIONS", true),
                enable_mock_llm: env_bool("GRID_FORGE_ENABLE_MOCK_LLM", true),
                enable_connector_writes: env_bool("GRID_FORGE_ENABLE_CONNECTOR_WRITES", false),
                require_human_approval_for_customer_drafts: env_bool(
                    "GRID_FORGE_REQUIRE_CUSTOMER_DRAFT_APPROVAL",
                    true,
                ),
            },
            retention_days: env_var("GRID_FORGE_RETENTION_DAYS", "365")
                .parse()
                .map_err(|err| format!("invalid GRID_FORGE_RETENTION_DAYS: {err}"))?,
        })
    }
}

fn env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(default)
}
