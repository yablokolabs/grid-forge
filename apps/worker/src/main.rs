use grid_forge_ai::{InMemoryVectorStore, VectorStore};
use grid_forge_auth::demo_organization_id;
use grid_forge_common::{telemetry, AppConfig};
use grid_forge_domain::{CopilotModule, SearchRequest};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    telemetry::init("grid-forge-worker");
    let _config = AppConfig::from_env().expect("load configuration");
    let org = demo_organization_id();
    let user = Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("demo user");
    let (store, documents) = InMemoryVectorStore::seed_demo(org, user)
        .await
        .expect("seed demo docs");

    tracing::info!(
        documents = documents.len(),
        "worker bootstrapped demo ingestion pipeline"
    );
    let smoke = store
        .search(
            org,
            SearchRequest {
                query: "outage restoration transformer vegetation regulatory".into(),
                module: Some(CopilotModule::Engineering),
                limit: Some(3),
                filters: json!({}),
            },
        )
        .await
        .expect("search demo chunks");
    tracing::info!(chunks = smoke.len(), "mock retrieval smoke test complete");

    if std::env::var("GRID_FORGE_WORKER_ONCE").unwrap_or_default() == "true" {
        return;
    }

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("worker received shutdown signal");
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                tracing::info!(
                    "worker heartbeat: ready for ingestion, embedding, and long-running agent jobs"
                );
            }
        }
    }
}
