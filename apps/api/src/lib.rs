use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use grid_forge_ai::{
    chunk_text, demo_document_body, CopilotService, InMemoryVectorStore, MockLlmProvider,
    VectorStore,
};
use grid_forge_audit::{AuditInput, AuditLogger, InMemoryAuditLogger};
use grid_forge_auth::{demo_organization_id, AuthContext, AuthService};
use grid_forge_common::{AppConfig, AppError};
use grid_forge_connectors::{ConnectorRegistration, MockConnectorRuntime};
use grid_forge_domain::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiState {
    pub config: AppConfig,
    pub auth: AuthService,
    pub audit: InMemoryAuditLogger,
    pub connector_runtime: MockConnectorRuntime,
    pub vector_store: Arc<InMemoryVectorStore>,
    pub copilot: CopilotService<MockLlmProvider, InMemoryVectorStore>,
    pub documents: Arc<RwLock<HashMap<DocumentId, Document>>>,
    pub agent_runs: Arc<RwLock<HashMap<AgentRunId, AgentRun>>>,
    pub connectors: Arc<RwLock<HashMap<Uuid, Connector>>>,
}

impl ApiState {
    pub async fn demo(config: AppConfig) -> Result<Self, AppError> {
        let auth = AuthService::new(config.auth.clone());
        let demo_user_id =
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("valid demo user");
        let (vector_store, documents) =
            InMemoryVectorStore::seed_demo(demo_organization_id(), demo_user_id).await?;
        let vector_store = Arc::new(vector_store);
        let copilot = CopilotService::new(
            Arc::new(MockLlmProvider),
            Arc::clone(&vector_store),
            config.feature_flags.clone(),
        );

        Ok(Self {
            config,
            auth,
            audit: InMemoryAuditLogger::new(),
            connector_runtime: MockConnectorRuntime,
            vector_store,
            copilot,
            documents: Arc::new(RwLock::new(
                documents
                    .into_iter()
                    .map(|document| (document.id, document))
                    .collect(),
            )),
            agent_runs: Arc::new(RwLock::new(HashMap::new())),
            connectors: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

pub fn build_router(state: ApiState) -> Router {
    let cors = if state.config.auth.demo_mode {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
    };

    Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/me", get(me))
        .route("/documents", post(upload_document))
        .route("/documents/:id/process", post(process_document))
        .route("/documents/:id", get(get_document))
        .route("/search", post(search))
        .route("/agent-runs", post(create_agent_run))
        .route("/agent-runs/:id", get(get_agent_run))
        .route("/audit-events", get(audit_events))
        .route("/connectors", post(create_connector))
        .route(
            "/customer-interactions/classify",
            post(classify_customer_interaction),
        )
        .route("/regulatory/draft", post(draft_regulatory))
        .route(
            "/engineering/outage-summary",
            post(engineering_outage_summary),
        )
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 1024)) // 1 MB
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        login,
        me,
        upload_document,
        process_document,
        get_document,
        search,
        create_agent_run,
        get_agent_run,
        audit_events,
        create_connector,
        classify_customer_interaction,
        draft_regulatory,
        engineering_outage_summary
    ),
    components(schemas(
        HealthResponse,
        LoginRequest,
        LoginResponse,
        MeResponse,
        UploadDocumentRequest,
        ProcessDocumentResponse,
        CreateAgentRunRequest,
        CreateConnectorRequest,
        CustomerClassifyRequest,
        RegulatoryDraftRequest,
        EngineeringOutageSummaryRequest,
        Organization,
        User,
        Role,
        Permission,
        UtilityAccount,
        Asset,
        WorkOrder,
        OutageEvent,
        MeterAlert,
        Document,
        DocumentChunk,
        Citation,
        Regulation,
        Filing,
        CustomerInteraction,
        AgentRun,
        PromptTemplate,
        Connector,
        AuditEvent,
        SearchRequest,
        SearchResponse,
        SearchResult,
        GroundedAnswer,
        ClassificationResult
    )),
    tags(
        (name = "auth", description = "Demo JWT auth and current-user lookup"),
        (name = "documents", description = "Document ingestion and retrieval metadata"),
        (name = "copilots", description = "Utility-focused AI copilot workflows"),
        (name = "audit", description = "Audit trail for AI actions and sensitive decisions")
    )
)]
struct ApiDoc;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub ok: bool,
    pub service: &'static str,
    pub demo_mode: bool,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthResponse)))]
async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "grid-forge-api",
        demo_mode: state.config.auth.demo_mode,
    })
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[utoipa::path(post, path = "/auth/login", request_body = LoginRequest, responses((status = 200, body = LoginResponse)), tag = "auth")]
async fn login(
    State(state): State<ApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let (user, token) = state.auth.login_demo(&request.email, &request.password)?;
    Ok(Json(LoginResponse { token, user }))
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    #[schema(value_type = String, format = Uuid)]
    pub user_id: UserId,
    #[schema(value_type = String, format = Uuid)]
    pub organization_id: OrganizationId,
    pub email: String,
    pub role: Role,
    pub permissions: Vec<Permission>,
}

#[utoipa::path(get, path = "/me", responses((status = 200, body = MeResponse)), security(("bearerAuth" = [])), tag = "auth")]
async fn me(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<MeResponse>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    Ok(Json(MeResponse {
        user_id: auth.user_id,
        organization_id: auth.organization_id,
        email: auth.email,
        role: auth.role,
        permissions: auth.role.permissions().to_vec(),
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadDocumentRequest {
    pub title: String,
    pub source_uri: String,
    pub document_type: DocumentType,
    pub tags: Vec<String>,
}

#[utoipa::path(post, path = "/documents", request_body = UploadDocumentRequest, responses((status = 200, body = Document)), security(("bearerAuth" = [])), tag = "documents")]
async fn upload_document(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<UploadDocumentRequest>,
) -> Result<Json<Document>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::IngestDocuments)?;
    let document = Document {
        id: Uuid::new_v4(),
        organization_id: auth.organization_id,
        title: request.title,
        source_uri: request.source_uri,
        document_type: request.document_type,
        status: DocumentStatus::Uploaded,
        tags: request.tags,
        uploaded_by: auth.user_id,
        created_at: Utc::now(),
    };
    state
        .documents
        .write()
        .await
        .insert(document.id, document.clone());
    Ok(Json(document))
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProcessDocumentResponse {
    pub document: Document,
    pub chunks_indexed: usize,
}

#[utoipa::path(post, path = "/documents/{id}/process", params(("id" = Uuid, Path)), responses((status = 200, body = ProcessDocumentResponse)), security(("bearerAuth" = [])), tag = "documents")]
async fn process_document(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProcessDocumentResponse>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::IngestDocuments)?;

    // Extract document data under a short-lived write lock (no .await while locked).
    let (title, source_uri) = {
        let mut documents = state.documents.write().await;
        let document = documents
            .get_mut(&id)
            .filter(|doc| doc.organization_id == auth.organization_id)
            .ok_or_else(|| AppError::NotFound(format!("document {id}")))?;
        document.status = DocumentStatus::Processing;
        (document.title.clone(), document.source_uri.clone())
    };

    let body = demo_document_body(&title);
    let fallback;
    let source_text = if body.is_empty() {
        fallback = format!(
            "{}\n\nUploaded source {} is queued for production parser integration.",
            title, source_uri
        );
        fallback.as_str()
    } else {
        body
    };

    // Re-acquire read lock to build chunks from the document snapshot.
    let document_snapshot = {
        let documents = state.documents.read().await;
        documents
            .get(&id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("document {id}")))?
    };
    let chunks = chunk_text(&document_snapshot, source_text);
    let chunks_indexed = chunks.len();

    // Async I/O outside any lock.
    state.vector_store.upsert_chunks(chunks).await?;

    // Final status update under a short-lived write lock.
    let document = {
        let mut documents = state.documents.write().await;
        let document = documents
            .get_mut(&id)
            .ok_or_else(|| AppError::NotFound(format!("document {id}")))?;
        document.status = DocumentStatus::Indexed;
        document.clone()
    };

    Ok(Json(ProcessDocumentResponse {
        document,
        chunks_indexed,
    }))
}

#[utoipa::path(get, path = "/documents/{id}", params(("id" = Uuid, Path)), responses((status = 200, body = Document)), security(("bearerAuth" = [])), tag = "documents")]
async fn get_document(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Document>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::ReadDocuments)?;
    let documents = state.documents.read().await;
    let document = documents
        .get(&id)
        .filter(|doc| doc.organization_id == auth.organization_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("document {id}")))?;
    Ok(Json(document))
}

#[utoipa::path(post, path = "/search", request_body = SearchRequest, responses((status = 200, body = SearchResponse)), security(("bearerAuth" = [])), tag = "documents")]
async fn search(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::ReadDocuments)?;
    let query = request.query.clone();
    let results = state
        .vector_store
        .search(auth.organization_id, request)
        .await?;
    Ok(Json(SearchResponse { query, results }))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRunRequest {
    pub module: CopilotModule,
    pub prompt: String,
    #[schema(value_type = String, format = Uuid)]
    pub prompt_template_id: Option<Uuid>,
    pub metadata: Value,
}

#[utoipa::path(post, path = "/agent-runs", request_body = CreateAgentRunRequest, responses((status = 200, body = AgentRun)), security(("bearerAuth" = [])), tag = "copilots")]
async fn create_agent_run(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<CreateAgentRunRequest>,
) -> Result<Json<AgentRun>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    require_module_permission(&auth, request.module)?;
    let approval_required =
        grid_forge_ai::prompt_requires_human_approval(request.module, &request.prompt);

    let output = if approval_required {
        None
    } else {
        let result = state
            .copilot
            .retrieve(auth.organization_id, request.prompt.clone(), request.module)
            .await?;
        Some(GroundedAnswer {
            answer: format!(
                "Queued {module:?} run completed with mock grounded retrieval.",
                module = request.module
            ),
            citations: result
                .results
                .into_iter()
                .map(|result| result.citation)
                .collect(),
            confidence: 0.74,
            review_needed: false,
            safety_notes: vec![
                "Generic agent runs should be narrowed into module-specific workflows.".into(),
            ],
        })
    };

    let agent_run = AgentRun {
        id: Uuid::new_v4(),
        organization_id: auth.organization_id,
        requested_by: auth.user_id,
        module: request.module,
        prompt_template_id: request.prompt_template_id,
        status: if approval_required {
            AgentRunStatus::WaitingForHumanApproval
        } else {
            AgentRunStatus::Completed
        },
        approval_state: if approval_required {
            ApprovalState::Required
        } else {
            ApprovalState::NotRequired
        },
        input: json!({"prompt": request.prompt, "metadata": request.metadata}),
        output: output.clone(),
        created_at: Utc::now(),
        completed_at: output.as_ref().map(|_| Utc::now()),
    };

    state
        .agent_runs
        .write()
        .await
        .insert(agent_run.id, agent_run.clone());
    state
        .audit
        .record(AuditInput {
            organization_id: auth.organization_id,
            actor_user_id: Some(auth.user_id),
            action: "agent_run.created".into(),
            resource_type: "agent_run".into(),
            resource_id: Some(agent_run.id),
            module: Some(agent_run.module),
            citations: output.map(|answer| answer.citations).unwrap_or_default(),
            decision: Some(format!("approval_state={:?}", agent_run.approval_state)),
            metadata: json!({"status": agent_run.status}),
        })
        .await?;
    Ok(Json(agent_run))
}

#[utoipa::path(get, path = "/agent-runs/{id}", params(("id" = Uuid, Path)), responses((status = 200, body = AgentRun)), security(("bearerAuth" = [])), tag = "copilots")]
async fn get_agent_run(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentRun>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::ReadDocuments)?;
    let runs = state.agent_runs.read().await;
    let run = runs
        .get(&id)
        .filter(|run| run.organization_id == auth.organization_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("agent run {id}")))?;
    Ok(Json(run))
}

#[utoipa::path(get, path = "/audit-events", responses((status = 200, body = [AuditEvent])), security(("bearerAuth" = [])), tag = "audit")]
async fn audit_events(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<Vec<AuditEvent>>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::ViewAuditEvents)?;
    Ok(Json(
        state.audit.list_for_org(auth.organization_id, 100).await?,
    ))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectorRequest {
    pub name: String,
    pub kind: ConnectorKind,
    pub config: Value,
}

#[utoipa::path(post, path = "/connectors", request_body = CreateConnectorRequest, responses((status = 200, body = Connector)), security(("bearerAuth" = [])))]
async fn create_connector(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<CreateConnectorRequest>,
) -> Result<Json<Connector>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::ManageConnectors)?;
    let connector = state.connector_runtime.register(ConnectorRegistration {
        organization_id: auth.organization_id,
        name: request.name,
        kind: request.kind,
        config: request.config,
    });
    state
        .connectors
        .write()
        .await
        .insert(connector.id, connector.clone());
    Ok(Json(connector))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomerClassifyRequest {
    pub raw_text: String,
    #[schema(value_type = String, format = Uuid)]
    pub account_id: Option<Uuid>,
}

#[utoipa::path(post, path = "/customer-interactions/classify", request_body = CustomerClassifyRequest, responses((status = 200, body = ClassificationResult)), security(("bearerAuth" = [])), tag = "copilots")]
async fn classify_customer_interaction(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<CustomerClassifyRequest>,
) -> Result<Json<ClassificationResult>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::RunCustomerOpsCopilot)?;
    let result = state
        .copilot
        .classify_customer_interaction(auth.organization_id, &request.raw_text)
        .await?;
    state
        .audit
        .record(AuditInput {
            organization_id: auth.organization_id,
            actor_user_id: Some(auth.user_id),
            action: "customer_interaction.classified".into(),
            resource_type: "customer_interaction".into(),
            resource_id: request.account_id,
            module: Some(CopilotModule::CustomerOps),
            citations: result.citations.clone(),
            decision: Some(format!(
                "issue_class={:?}; review_needed={}",
                result.issue_class, result.review_needed
            )),
            metadata: json!({"pii_policy":"mask-before-provider-call"}),
        })
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegulatoryDraftRequest {
    pub docket_number: String,
    pub question: String,
}

#[utoipa::path(post, path = "/regulatory/draft", request_body = RegulatoryDraftRequest, responses((status = 200, body = GroundedAnswer)), security(("bearerAuth" = [])), tag = "copilots")]
async fn draft_regulatory(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<RegulatoryDraftRequest>,
) -> Result<Json<GroundedAnswer>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::RunRegulatoryCopilot)?;
    let answer = state
        .copilot
        .regulatory_draft(
            auth.organization_id,
            &request.docket_number,
            &request.question,
        )
        .await?;
    audit_grounded_answer(
        &state,
        &auth,
        CopilotModule::Regulatory,
        "regulatory.draft",
        &answer,
    )
    .await?;
    Ok(Json(answer))
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringOutageSummaryRequest {
    pub outage_number: String,
    pub field_notes: String,
}

#[utoipa::path(post, path = "/engineering/outage-summary", request_body = EngineeringOutageSummaryRequest, responses((status = 200, body = GroundedAnswer)), security(("bearerAuth" = [])), tag = "copilots")]
async fn engineering_outage_summary(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(request): Json<EngineeringOutageSummaryRequest>,
) -> Result<Json<GroundedAnswer>, ApiError> {
    let auth = auth_from_headers(&state, &headers)?;
    auth.require(Permission::RunEngineeringCopilot)?;
    let answer = state
        .copilot
        .engineering_outage_summary(
            auth.organization_id,
            &request.outage_number,
            &request.field_notes,
        )
        .await?;
    audit_grounded_answer(
        &state,
        &auth,
        CopilotModule::Engineering,
        "engineering.outage_summary",
        &answer,
    )
    .await?;
    Ok(Json(answer))
}

async fn audit_grounded_answer(
    state: &ApiState,
    auth: &AuthContext,
    module: CopilotModule,
    action: &str,
    answer: &GroundedAnswer,
) -> Result<(), ApiError> {
    state
        .audit
        .record(AuditInput {
            organization_id: auth.organization_id,
            actor_user_id: Some(auth.user_id),
            action: action.into(),
            resource_type: "grounded_answer".into(),
            resource_id: None,
            module: Some(module),
            citations: answer.citations.clone(),
            decision: Some(format!(
                "confidence={:.2}; review_needed={}",
                answer.confidence, answer.review_needed
            )),
            metadata: json!({"citation_count": answer.citations.len()}),
        })
        .await?;
    Ok(())
}

fn auth_from_headers(state: &ApiState, headers: &HeaderMap) -> Result<AuthContext, ApiError> {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    Ok(state.auth.verify_bearer(header)?)
}

fn require_module_permission(auth: &AuthContext, module: CopilotModule) -> Result<(), AppError> {
    match module {
        CopilotModule::Engineering => auth.require(Permission::RunEngineeringCopilot),
        CopilotModule::Regulatory => auth.require(Permission::RunRegulatoryCopilot),
        CopilotModule::CustomerOps => auth.require(Permission::RunCustomerOpsCopilot),
    }
}

#[derive(Debug)]
pub struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Config(_) | AppError::External(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let body = Json(json!({"error": self.0.to_string()}));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_route_works() {
        let config = AppConfig::from_env().unwrap();
        let app = build_router(ApiState::demo(config).await.unwrap());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
