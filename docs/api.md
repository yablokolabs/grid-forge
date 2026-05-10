# API

OpenAPI is served at `/docs` when `apps/api` is running.

## Authentication

`POST /auth/login`

```json
{
  "email": "engineer@cedar-rapids.example",
  "password": "demo-password"
}
```

Returns a JWT. Send it as:

```http
Authorization: Bearer <token>
```

## Documents

- `POST /documents` — create document metadata.
- `POST /documents/:id/process` — run demo chunking/indexing.
- `GET /documents/:id` — fetch document metadata.
- `POST /search` — retrieve chunks plus citations.

## Copilots

- `POST /engineering/outage-summary`
- `POST /regulatory/draft`
- `POST /customer-interactions/classify`
- `POST /agent-runs`
- `GET /agent-runs/:id`

All grounded outputs include citation arrays. Customer-facing drafts and finalization-like prompts are marked review-needed or approval-required.

## Audit

- `GET /audit-events`

Requires `ViewAuditEvents`. Audit records include actor, module, resource, citations, and decision metadata.

## Connectors

- `POST /connectors`

Creates a redacted connector config record. Real connector reads/writes are intentionally isolated behind traits and feature flags.
