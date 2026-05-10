use async_trait::async_trait;
use chrono::Utc;
use grid_forge_common::AppResult;
use grid_forge_domain::{AuditEvent, Citation, CopilotModule, OrganizationId, UserId};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuditInput {
    pub organization_id: OrganizationId,
    pub actor_user_id: Option<UserId>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub module: Option<CopilotModule>,
    pub citations: Vec<Citation>,
    pub decision: Option<String>,
    pub metadata: Value,
}

#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn record(&self, input: AuditInput) -> AppResult<AuditEvent>;
    async fn list_for_org(
        &self,
        organization_id: OrganizationId,
        limit: usize,
    ) -> AppResult<Vec<AuditEvent>>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryAuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl InMemoryAuditLogger {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AuditLogger for InMemoryAuditLogger {
    async fn record(&self, input: AuditInput) -> AppResult<AuditEvent> {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            organization_id: input.organization_id,
            actor_user_id: input.actor_user_id,
            action: input.action,
            resource_type: input.resource_type,
            resource_id: input.resource_id,
            module: input.module,
            citations: input.citations,
            decision: input.decision,
            metadata: input.metadata,
            created_at: Utc::now(),
        };
        self.events.write().await.push(event.clone());
        Ok(event)
    }

    async fn list_for_org(
        &self,
        organization_id: OrganizationId,
        limit: usize,
    ) -> AppResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|event| event.organization_id == organization_id)
            .rev()
            .take(limit)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid_forge_auth::demo_organization_id;
    use serde_json::json;

    #[tokio::test]
    async fn audit_logger_records_ai_action() {
        let logger = InMemoryAuditLogger::new();
        let org = demo_organization_id();
        logger
            .record(AuditInput {
                organization_id: org,
                actor_user_id: None,
                action: "agent_run.completed".into(),
                resource_type: "agent_run".into(),
                resource_id: None,
                module: Some(CopilotModule::Engineering),
                citations: vec![],
                decision: Some("review_needed=false".into()),
                metadata: json!({"test": true}),
            })
            .await
            .expect("record audit event");

        assert_eq!(logger.list_for_org(org, 10).await.unwrap().len(), 1);
    }
}
