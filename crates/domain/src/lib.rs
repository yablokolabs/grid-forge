use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

pub type OrganizationId = Uuid;
pub type UserId = Uuid;
pub type DocumentId = Uuid;
pub type AgentRunId = Uuid;
pub type AuditEventId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Demo,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    OpsManager,
    UtilityEngineer,
    RegulatoryAnalyst,
    CustomerOps,
    Auditor,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ReadDocuments,
    IngestDocuments,
    RunEngineeringCopilot,
    RunRegulatoryCopilot,
    RunCustomerOpsCopilot,
    ManageConnectors,
    ViewAuditEvents,
    ApproveSensitiveActions,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            Role::Admin => &[
                ReadDocuments,
                IngestDocuments,
                RunEngineeringCopilot,
                RunRegulatoryCopilot,
                RunCustomerOpsCopilot,
                ManageConnectors,
                ViewAuditEvents,
                ApproveSensitiveActions,
            ],
            Role::OpsManager => &[
                ReadDocuments,
                RunEngineeringCopilot,
                RunCustomerOpsCopilot,
                ViewAuditEvents,
                ApproveSensitiveActions,
            ],
            Role::UtilityEngineer => &[ReadDocuments, RunEngineeringCopilot],
            Role::RegulatoryAnalyst => &[ReadDocuments, IngestDocuments, RunRegulatoryCopilot],
            Role::CustomerOps => &[ReadDocuments, RunCustomerOpsCopilot],
            Role::Auditor => &[ReadDocuments, ViewAuditEvents],
            Role::ReadOnly => &[ReadDocuments],
        }
    }

    pub fn has_permission(self, permission: Permission) -> bool {
        self.permissions().contains(&permission)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    #[schema(value_type = String, format = Uuid)]
    pub id: OrganizationId,
    pub name: String,
    pub utility_type: UtilityType,
    pub service_territory: String,
    pub demo_mode: bool,
    pub retention_days: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UtilityType {
    InvestorOwned,
    Cooperative,
    Municipal,
    PublicPower,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[schema(value_type = String, format = Uuid)]
    pub id: UserId,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtilityAccount {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub account_number: String,
    pub service_address: String,
    pub feeder_id: Option<String>,
    pub meter_id: Option<String>,
    pub pii_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub external_id: String,
    pub asset_type: AssetType,
    pub name: String,
    pub feeder_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub health_index: Option<f32>,
    pub risk_notes: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Transformer,
    Feeder,
    Breaker,
    Recloser,
    Pole,
    Substation,
    Meter,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrder {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub external_id: String,
    #[schema(value_type = String, format = Uuid)]
    pub asset_id: Option<Uuid>,
    pub title: String,
    pub status: WorkOrderStatus,
    pub priority: Priority,
    pub assigned_crew: Option<String>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkOrderStatus {
    Open,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutageEvent {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub outage_number: String,
    pub feeder_id: Option<String>,
    pub affected_customers: u32,
    pub started_at: DateTime<Utc>,
    pub estimated_restore_at: Option<DateTime<Utc>>,
    pub cause: Option<String>,
    pub status: OutageStatus,
    pub crew_status: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutageStatus {
    New,
    Investigating,
    CrewAssigned,
    Restoring,
    Restored,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeterAlert {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub meter_id: String,
    #[schema(value_type = String, format = Uuid)]
    pub account_id: Option<Uuid>,
    pub alert_type: MeterAlertType,
    pub severity: Priority,
    pub observed_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeterAlertType {
    LastGasp,
    VoltageSag,
    Tamper,
    UsageAnomaly,
    PowerQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[schema(value_type = String, format = Uuid)]
    pub id: DocumentId,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub title: String,
    pub source_uri: String,
    pub document_type: DocumentType,
    pub status: DocumentStatus,
    pub tags: Vec<String>,
    #[schema(value_type = String, format = Uuid)]
    pub uploaded_by: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    EngineeringManual,
    Sop,
    RegulatoryOrder,
    Filing,
    CustomerComplaintSet,
    WorkOrderExport,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Uploaded,
    Processing,
    Indexed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    #[schema(value_type = String, format = Uuid)]
    pub document_id: DocumentId,
    pub chunk_index: i32,
    pub text: String,
    pub token_count: u32,
    pub section: Option<String>,
    pub page_start: Option<u32>,
    pub page_end: Option<u32>,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    #[schema(value_type = String, format = Uuid)]
    pub document_id: DocumentId,
    #[schema(value_type = String, format = Uuid)]
    pub chunk_id: Option<Uuid>,
    pub title: String,
    pub section: Option<String>,
    pub page_start: Option<u32>,
    pub page_end: Option<u32>,
    pub source_uri: String,
    pub quote: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Regulation {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub jurisdiction: String,
    pub docket_number: Option<String>,
    pub title: String,
    pub effective_date: Option<DateTime<Utc>>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Filing {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    #[schema(value_type = String, format = Uuid)]
    pub regulation_id: Option<Uuid>,
    pub docket_number: String,
    pub title: String,
    pub filed_at: Option<DateTime<Utc>>,
    pub status: FilingStatus,
    #[schema(value_type = String, format = Uuid)]
    pub document_id: Option<DocumentId>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Draft,
    Filed,
    AwaitingResponse,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInteraction {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    #[schema(value_type = String, format = Uuid)]
    pub account_id: Option<Uuid>,
    pub channel: InteractionChannel,
    pub raw_text: String,
    pub received_at: DateTime<Utc>,
    pub classification: Option<CustomerIssueClass>,
    pub pii_detected: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InteractionChannel {
    Phone,
    Email,
    Chat,
    WebForm,
    Social,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CustomerIssueClass {
    Outage,
    Billing,
    NewService,
    Vegetation,
    PowerQuality,
    Safety,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CopilotModule {
    Engineering,
    Regulatory,
    CustomerOps,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Queued,
    Running,
    WaitingForHumanApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    NotRequired,
    Required,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    #[schema(value_type = String, format = Uuid)]
    pub id: AgentRunId,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    #[schema(value_type = String, format = Uuid)]
    pub requested_by: UserId,
    pub module: CopilotModule,
    #[schema(value_type = String, format = Uuid)]
    pub prompt_template_id: Option<Uuid>,
    pub status: AgentRunStatus,
    pub approval_state: ApprovalState,
    pub input: Value,
    pub output: Option<GroundedAnswer>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundedAnswer {
    pub answer: String,
    pub citations: Vec<Citation>,
    pub confidence: f32,
    pub review_needed: bool,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub module: CopilotModule,
    pub name: String,
    pub template: String,
    pub requires_human_approval: bool,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Connector {
    #[schema(value_type = String, format = Uuid)]
    pub id: Uuid,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub name: String,
    pub kind: ConnectorKind,
    pub status: ConnectorStatus,
    pub config: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    Gis,
    Scada,
    AmiMdms,
    Oms,
    Cis,
    Eam,
    Ticketing,
    ObjectStorage,
    DemoFileDrop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorStatus {
    Draft,
    Active,
    Paused,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    #[schema(value_type = String, format = Uuid)]
    pub id: AuditEventId,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    #[schema(value_type = String, format = Uuid)]
    pub actor_user_id: Option<UserId>,
    pub action: String,
    pub resource_type: String,
    #[schema(value_type = String, format = Uuid)]
    pub resource_id: Option<Uuid>,
    pub module: Option<CopilotModule>,
    pub citations: Vec<Citation>,
    pub decision: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub module: Option<CopilotModule>,
    pub limit: Option<usize>,
    pub filters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub chunk: DocumentChunk,
    pub citation: Citation,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationResult {
    pub issue_class: CustomerIssueClass,
    pub confidence: f32,
    pub summary: String,
    pub suggested_escalation_path: String,
    pub draft_response: String,
    pub citations: Vec<Citation>,
    pub review_needed: bool,
}
