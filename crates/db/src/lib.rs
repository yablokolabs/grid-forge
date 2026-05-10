use async_trait::async_trait;
use grid_forge_common::{AppError, AppResult, DatabaseConfig, ObjectStorageConfig};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub async fn connect(config: &DatabaseConfig) -> AppResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|err| AppError::External(format!("postgres connection failed: {err}")))
}

pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|err| AppError::External(format!("migration failed: {err}")))
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> AppResult<String>;
    async fn get(&self, key: &str) -> AppResult<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct S3ObjectStoreConfig {
    pub endpoint: String,
    pub bucket: String,
}

impl From<&ObjectStorageConfig> for S3ObjectStoreConfig {
    fn from(value: &ObjectStorageConfig) -> Self {
        Self {
            endpoint: value.endpoint.clone(),
            bucket: value.bucket.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryObjectStore {
    objects: Arc<RwLock<HashMap<String, StoredObject>>>,
}

#[derive(Debug, Clone)]
struct StoredObject {
    bytes: Vec<u8>,
    content_type: String,
}

#[async_trait]
impl ObjectStore for InMemoryObjectStore {
    async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> AppResult<String> {
        self.objects.write().await.insert(
            key.to_string(),
            StoredObject {
                bytes,
                content_type: content_type.to_string(),
            },
        );
        Ok(format!("memory://{key}"))
    }

    async fn get(&self, key: &str) -> AppResult<Vec<u8>> {
        self.objects
            .read()
            .await
            .get(key)
            .map(|object| object.bytes.clone())
            .ok_or_else(|| AppError::NotFound(format!("object {key}")))
    }
}

impl InMemoryObjectStore {
    pub async fn content_type(&self, key: &str) -> Option<String> {
        self.objects
            .read()
            .await
            .get(key)
            .map(|object| object.content_type.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_object_store_round_trip() {
        let store = InMemoryObjectStore::default();
        let uri = store
            .put("docs/outage.md", b"outage SOP".to_vec(), "text/markdown")
            .await
            .unwrap();
        assert_eq!(uri, "memory://docs/outage.md");
        assert_eq!(
            store.get("docs/outage.md").await.unwrap(),
            b"outage SOP".to_vec()
        );
        assert_eq!(
            store.content_type("docs/outage.md").await.unwrap(),
            "text/markdown"
        );
    }
}
