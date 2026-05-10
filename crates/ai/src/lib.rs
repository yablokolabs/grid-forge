use async_trait::async_trait;
use chrono::Utc;
use grid_forge_common::{AppError, AppResult, FeatureFlags};
use grid_forge_domain::{
    Citation, ClassificationResult, CopilotModule, CustomerIssueClass, Document, DocumentChunk,
    DocumentStatus, DocumentType, GroundedAnswer, OrganizationId, SearchRequest, SearchResponse,
    SearchResult, UserId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cmp::Ordering, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> AppResult<LlmResponse>;
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, input: &str) -> AppResult<Vec<f32>>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert_chunks(&self, chunks: Vec<DocumentChunk>) -> AppResult<()>;
    async fn search(
        &self,
        organization_id: OrganizationId,
        request: SearchRequest,
    ) -> AppResult<Vec<SearchResult>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmRequest {
    pub module: CopilotModule,
    pub task: String,
    pub prompt: String,
    pub retrieved_context: Vec<Citation>,
    pub require_citations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmResponse {
    pub answer: String,
    pub confidence: f32,
    pub review_needed: bool,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MockLlmProvider;

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, request: LlmRequest) -> AppResult<LlmResponse> {
        let citation_count = request.retrieved_context.len();
        let prefix = match request.module {
            CopilotModule::Engineering => "Engineering copilot summary",
            CopilotModule::Regulatory => "Regulatory copilot draft",
            CopilotModule::CustomerOps => "Customer operations draft",
        };

        let review_needed = matches!(request.module, CopilotModule::CustomerOps)
            || request.prompt.to_lowercase().contains("finalize")
            || citation_count == 0;

        Ok(LlmResponse {
            answer: format!(
                "{prefix}: {}. Grounded in {citation_count} retrieved source(s). Review operational assumptions before external use.",
                request.task
            ),
            confidence: if citation_count >= 2 { 0.86 } else { 0.63 },
            review_needed,
            safety_notes: vec![
                "Do not execute switching, dispatch, billing, or regulatory submission actions without human approval.".into(),
                "Citations are required for document-grounded answers.".into(),
            ],
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, input: &str) -> AppResult<Vec<f32>> {
        let mut vector = vec![0.0; 8];
        for (index, byte) in input.bytes().enumerate() {
            vector[index % 8] += byte as f32 / 255.0;
        }
        Ok(vector)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryVectorStore {
    chunks: Arc<RwLock<Vec<DocumentChunk>>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn seed_demo(
        organization_id: OrganizationId,
        uploaded_by: UserId,
    ) -> AppResult<(Self, Vec<Document>)> {
        let store = Self::new();
        let documents = demo_documents(organization_id, uploaded_by);
        let mut chunks = Vec::new();
        for document in &documents {
            let body = demo_document_body(&document.title);
            chunks.extend(chunk_text(document, body));
        }
        store.upsert_chunks(chunks).await?;
        Ok((store, documents))
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert_chunks(&self, chunks: Vec<DocumentChunk>) -> AppResult<()> {
        self.chunks.write().await.extend(chunks);
        Ok(())
    }

    async fn search(
        &self,
        organization_id: OrganizationId,
        request: SearchRequest,
    ) -> AppResult<Vec<SearchResult>> {
        let query_terms = tokenize(&request.query);
        let limit = request.limit.unwrap_or(5).clamp(1, 20);
        let chunks = self.chunks.read().await;
        let mut scored: Vec<SearchResult> = chunks
            .iter()
            .filter(|chunk| chunk.organization_id == organization_id)
            .filter_map(|chunk| {
                let score = lexical_score(&query_terms, &chunk.text);
                if score <= 0.0 {
                    return None;
                }
                Some(SearchResult {
                    citation: Citation {
                        document_id: chunk.document_id,
                        chunk_id: Some(chunk.id),
                        title: format!("{}", chunk.document_id),
                        section: chunk.section.clone(),
                        page_start: chunk.page_start,
                        page_end: chunk.page_end,
                        source_uri: format!("object://documents/{}", chunk.document_id),
                        quote: chunk.text.chars().take(260).collect(),
                        score,
                    },
                    chunk: chunk.clone(),
                    score,
                })
            })
            .collect();
        scored.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
        });
        scored.truncate(limit);
        Ok(scored)
    }
}

#[derive(Clone)]
pub struct CopilotService<L, V> {
    llm: Arc<L>,
    vector_store: Arc<V>,
    feature_flags: FeatureFlags,
}

impl<L, V> CopilotService<L, V>
where
    L: LlmProvider,
    V: VectorStore,
{
    pub fn new(llm: Arc<L>, vector_store: Arc<V>, feature_flags: FeatureFlags) -> Self {
        Self {
            llm,
            vector_store,
            feature_flags,
        }
    }

    pub async fn engineering_outage_summary(
        &self,
        organization_id: OrganizationId,
        outage_number: &str,
        field_notes: &str,
    ) -> AppResult<GroundedAnswer> {
        let search = self
            .retrieve(
                organization_id,
                format!("outage restoration safety switching crew communications {outage_number} {field_notes}"),
                CopilotModule::Engineering,
            )
            .await?;
        self.grounded_completion(
            CopilotModule::Engineering,
            "Summarize outage triage, likely operational risks, and next safe review steps.",
            field_notes,
            search.results,
        )
        .await
    }

    pub async fn regulatory_draft(
        &self,
        organization_id: OrganizationId,
        docket_number: &str,
        question: &str,
    ) -> AppResult<GroundedAnswer> {
        let search = self
            .retrieve(
                organization_id,
                format!("regulatory order filing docket {docket_number} {question}"),
                CopilotModule::Regulatory,
            )
            .await?;
        self.grounded_completion(
            CopilotModule::Regulatory,
            "Draft a regulator-facing response memo with source citations and caveats.",
            question,
            search.results,
        )
        .await
    }

    pub async fn classify_customer_interaction(
        &self,
        organization_id: OrganizationId,
        raw_text: &str,
    ) -> AppResult<ClassificationResult> {
        let lowered = raw_text.to_lowercase();
        let issue_class = if lowered.contains("outage") || lowered.contains("power") {
            CustomerIssueClass::Outage
        } else if lowered.contains("bill") || lowered.contains("charge") {
            CustomerIssueClass::Billing
        } else if lowered.contains("tree") || lowered.contains("vegetation") {
            CustomerIssueClass::Vegetation
        } else if lowered.contains("flicker") || lowered.contains("voltage") {
            CustomerIssueClass::PowerQuality
        } else {
            CustomerIssueClass::Unknown
        };

        let search = self
            .retrieve(
                organization_id,
                format!(
                    "customer explanation {issue_class:?} outage billing vegetation escalation"
                ),
                CopilotModule::CustomerOps,
            )
            .await?;
        let answer = self
            .grounded_completion(
                CopilotModule::CustomerOps,
                "Draft a customer-safe explanation and escalation recommendation.",
                raw_text,
                search.results,
            )
            .await?;

        Ok(ClassificationResult {
            issue_class,
            confidence: answer.confidence,
            summary: format!(
                "Classified as {issue_class:?}; PII should be masked before model calls."
            ),
            suggested_escalation_path: match issue_class {
                CustomerIssueClass::Outage => {
                    "OMS queue → dispatch desk → customer callback when ETR changes".into()
                }
                CustomerIssueClass::Billing => "CIS billing analyst review".into(),
                CustomerIssueClass::Vegetation => "Vegetation management inspection queue".into(),
                CustomerIssueClass::PowerQuality => "Power quality engineer review".into(),
                _ => "Customer operations supervisor triage".into(),
            },
            draft_response: answer.answer,
            citations: answer.citations,
            review_needed: true,
        })
    }

    pub async fn retrieve(
        &self,
        organization_id: OrganizationId,
        query: String,
        module: CopilotModule,
    ) -> AppResult<SearchResponse> {
        let results = self
            .vector_store
            .search(
                organization_id,
                SearchRequest {
                    query: query.clone(),
                    module: Some(module),
                    limit: Some(5),
                    filters: json!({}),
                },
            )
            .await?;
        Ok(SearchResponse { query, results })
    }

    async fn grounded_completion(
        &self,
        module: CopilotModule,
        task: &str,
        prompt: &str,
        results: Vec<SearchResult>,
    ) -> AppResult<GroundedAnswer> {
        let citations: Vec<Citation> = results.into_iter().map(|result| result.citation).collect();
        if self.feature_flags.require_citations && citations.is_empty() {
            return Err(AppError::Validation(
                "grounded AI response requires at least one citation".into(),
            ));
        }
        let llm = self
            .llm
            .complete(LlmRequest {
                module,
                task: task.to_string(),
                prompt: prompt.to_string(),
                retrieved_context: citations.clone(),
                require_citations: self.feature_flags.require_citations,
            })
            .await?;
        Ok(GroundedAnswer {
            answer: llm.answer,
            citations,
            confidence: llm.confidence,
            review_needed: llm.review_needed,
            safety_notes: llm.safety_notes,
        })
    }
}

pub fn prompt_requires_human_approval(module: CopilotModule, prompt_name: &str) -> bool {
    matches!(module, CopilotModule::CustomerOps)
        || prompt_name.contains("final")
        || prompt_name.contains("submit")
        || prompt_name.contains("dispatch")
}

pub fn chunk_text(document: &Document, text: &str) -> Vec<DocumentChunk> {
    text.split("\n\n")
        .enumerate()
        .filter(|(_, section)| !section.trim().is_empty())
        .map(|(index, section)| DocumentChunk {
            id: Uuid::new_v4(),
            organization_id: document.organization_id,
            document_id: document.id,
            chunk_index: index as i32,
            text: section.trim().to_string(),
            token_count: section.split_whitespace().count() as u32,
            section: Some(format!("section-{}", index + 1)),
            page_start: Some((index + 1) as u32),
            page_end: Some((index + 1) as u32),
            embedding_model: Some("mock-lexical-v0".into()),
        })
        .collect()
}

pub fn demo_documents(organization_id: OrganizationId, uploaded_by: UserId) -> Vec<Document> {
    let now = Utc::now();
    [
        (
            "Outage Restoration SOP",
            "s3://demo/outage-sop.md",
            DocumentType::Sop,
        ),
        (
            "Vegetation Management Policy",
            "s3://demo/vegetation-policy.md",
            DocumentType::EngineeringManual,
        ),
        (
            "Transformer Maintenance Guide",
            "s3://demo/transformer-maintenance.md",
            DocumentType::EngineeringManual,
        ),
        (
            "Sample Regulatory Order 24-017",
            "s3://demo/regulatory-order-24-017.md",
            DocumentType::RegulatoryOrder,
        ),
        (
            "Customer Complaint Samples",
            "s3://demo/customer-complaints.md",
            DocumentType::CustomerComplaintSet,
        ),
    ]
    .into_iter()
    .map(|(title, source_uri, document_type)| Document {
        id: Uuid::new_v4(),
        organization_id,
        title: title.into(),
        source_uri: source_uri.into(),
        document_type,
        status: DocumentStatus::Indexed,
        tags: vec!["demo".into()],
        uploaded_by,
        created_at: now,
    })
    .collect()
}

pub fn demo_document_body(title: &str) -> &'static str {
    match title {
        "Outage Restoration SOP" => include_str!("../../../examples/mock-documents/outage-sop.md"),
        "Vegetation Management Policy" => {
            include_str!("../../../examples/mock-documents/vegetation-policy.md")
        }
        "Transformer Maintenance Guide" => {
            include_str!("../../../examples/mock-documents/transformer-maintenance-guide.md")
        }
        "Sample Regulatory Order 24-017" => {
            include_str!("../../../examples/mock-documents/sample-regulatory-order.md")
        }
        "Customer Complaint Samples" => {
            include_str!("../../../examples/mock-documents/customer-complaints.md")
        }
        _ => "",
    }
}

fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| term.len() > 2)
        .map(|term| term.to_lowercase())
        .collect()
}

fn lexical_score(query_terms: &[String], text: &str) -> f32 {
    let mut counts = HashMap::new();
    for term in tokenize(text) {
        *counts.entry(term).or_insert(0_u32) += 1;
    }
    query_terms
        .iter()
        .map(|term| counts.get(term).copied().unwrap_or_default() as f32)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use grid_forge_auth::demo_organization_id;

    fn flags() -> FeatureFlags {
        FeatureFlags {
            require_citations: true,
            enable_mock_llm: true,
            enable_connector_writes: false,
            require_human_approval_for_customer_drafts: true,
        }
    }

    #[tokio::test]
    async fn retrieval_returns_citations_for_outage_sop() {
        let org = demo_organization_id();
        let user = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let (store, _docs) = InMemoryVectorStore::seed_demo(org, user).await.unwrap();
        let results = store
            .search(
                org,
                SearchRequest {
                    query: "outage restoration crew safety".into(),
                    module: Some(CopilotModule::Engineering),
                    limit: Some(3),
                    filters: json!({}),
                },
            )
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results[0].citation.score > 0.0);
    }

    #[tokio::test]
    async fn customer_ops_classification_requires_review() {
        let org = demo_organization_id();
        let user = Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap();
        let (store, _docs) = InMemoryVectorStore::seed_demo(org, user).await.unwrap();
        let service = CopilotService::new(Arc::new(MockLlmProvider), Arc::new(store), flags());
        let result = service
            .classify_customer_interaction(
                org,
                "My power has been out for 3 hours and I need an update",
            )
            .await
            .unwrap();
        assert!(matches!(result.issue_class, CustomerIssueClass::Outage));
        assert!(result.review_needed);
        assert!(!result.citations.is_empty());
    }
}
