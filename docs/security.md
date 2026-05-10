# Security and enterprise posture

## Multi-tenancy

Every primary domain object carries `organization_id`. Production repositories should enforce tenant filters in every query and add row-level security where possible.

## RBAC

Roles map to permissions in `crates/domain`. API routes require module-specific permissions before running copilots.

## Audit logging

Every AI action records an `AuditEvent` with actor, module, resource, citations, review decision, and metadata. Production deployments should write audit events to append-only storage or export to a SIEM.

## PII handling

Customer interactions are treated as PII-bearing inputs. Production pipelines should mask account numbers, phone numbers, addresses, and names before model provider calls. Retention is configurable by tenant.

## Human-in-the-loop

Customer-facing drafts, finalization prompts, dispatch-like prompts, and submission-like prompts require review or approval. This repo intentionally does not automate switching, dispatch, billing adjustments, or regulatory filings.

## Secrets

No production secrets are hardcoded. Use environment variables or a managed secret store. Connector configs are redacted at registration boundaries.

## SOC2 roadmap

Controls to add later:

- Access reviews and SSO enforcement
- Change management evidence from CI/CD
- Incident response runbooks
- Vendor and model provider risk review
- Audit log retention and immutability
- Data classification and deletion workflows
- Tenant isolation tests and penetration testing
