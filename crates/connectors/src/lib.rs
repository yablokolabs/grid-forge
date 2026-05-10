use async_trait::async_trait;
use chrono::Utc;
use grid_forge_common::{AppError, AppResult};
use grid_forge_domain::{Connector, ConnectorKind, ConnectorStatus, OrganizationId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorRegistration {
    pub organization_id: OrganizationId,
    pub name: String,
    pub kind: ConnectorKind,
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorHealth {
    pub connector_id: Uuid,
    pub ok: bool,
    pub message: String,
}

#[async_trait]
pub trait UtilityConnector: Send + Sync {
    async fn health_check(&self, connector: &Connector) -> AppResult<ConnectorHealth>;
    async fn read_sample(&self, connector: &Connector) -> AppResult<Value>;
}

#[derive(Debug, Default, Clone)]
pub struct MockConnectorRuntime;

impl MockConnectorRuntime {
    pub fn register(&self, registration: ConnectorRegistration) -> Connector {
        Connector {
            id: Uuid::new_v4(),
            organization_id: registration.organization_id,
            name: registration.name,
            kind: registration.kind,
            status: ConnectorStatus::Draft,
            config: redact_connector_config(registration.config),
            created_at: Utc::now(),
        }
    }
}

#[async_trait]
impl UtilityConnector for MockConnectorRuntime {
    async fn health_check(&self, connector: &Connector) -> AppResult<ConnectorHealth> {
        Ok(ConnectorHealth {
            connector_id: connector.id,
            ok: true,
            message: format!(
                "mock {:?} connector reachable; no production writes enabled",
                connector.kind
            ),
        })
    }

    async fn read_sample(&self, connector: &Connector) -> AppResult<Value> {
        let sample = match connector.kind {
            ConnectorKind::Gis => serde_json::json!({"feeder":"F-12","assets":3}),
            ConnectorKind::AmiMdms => serde_json::json!({"lastGaspAlerts":7,"voltageSags":2}),
            ConnectorKind::Oms => serde_json::json!({"activeOutages":1,"affectedCustomers":184}),
            ConnectorKind::Scada => {
                return Err(AppError::External(
                    "SCADA read is intentionally mocked; configure a historian adapter".into(),
                ))
            }
            _ => serde_json::json!({"records":1}),
        };
        Ok(sample)
    }
}

pub fn redact_connector_config(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        for key in ["password", "secret", "token", "apiKey", "clientSecret"] {
            if object.contains_key(key) {
                object.insert(key.to_string(), Value::String("***redacted***".into()));
            }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connector_config_redacts_common_secret_names() {
        let redacted = redact_connector_config(serde_json::json!({
            "endpoint":"https://gis.example",
            "apiKey":"secret-key",
            "token":"secret-token"
        }));
        assert_eq!(redacted["apiKey"], "***redacted***");
        assert_eq!(redacted["token"], "***redacted***");
    }
}
