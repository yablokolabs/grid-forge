# Domain model

Core tenancy and security:

- `Organization` — tenant boundary for each utility or pilot customer.
- `User` — human actor with role-scoped permissions.
- `Role` / `Permission` — RBAC layer used by API routes.
- `AuditEvent` — immutable operational history for AI and sensitive actions.

Utility operations:

- `UtilityAccount` — customer account with PII-aware tags.
- `Asset` — transformer, feeder, breaker, recloser, pole, substation, meter.
- `WorkOrder` — maintenance or field task.
- `OutageEvent` — OMS-style outage event.
- `MeterAlert` — AMI/MDMS alert such as last-gasp, voltage sag, tamper, anomaly.

Documents and knowledge:

- `Document` — uploaded source item.
- `DocumentChunk` — retrievable text segment with optional embedding metadata.
- `Citation` — quote and location used to ground an AI answer.
- `Regulation` and `Filing` — regulatory corpus entities.

AI workflow:

- `PromptTemplate` — versioned module-specific prompt with approval policy.
- `AgentRun` — execution record for copilot requests.
- `GroundedAnswer` — answer, citations, confidence, review flag, safety notes.
- `CustomerInteraction` and `ClassificationResult` — customer ops classification flow.
- `Connector` — configured integration boundary for GIS, SCADA, AMI/MDMS, OMS, CIS, EAM.
