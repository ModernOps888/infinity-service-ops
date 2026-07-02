# Changelog

All notable changes to Infinity Service Ops are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-07-02

### Security

Following a deep security assessment of the scaffold, the following access-control and
data-egress vulnerabilities were fixed:

- **Cross-tenant RBAC bypass (HIGH):** `AuthorizationContext::allows` granted permissions
  from any role with a matching id, ignoring the role's `tenant_id`. Authorization now only
  considers roles scoped to the acting user's tenant, restoring tenant isolation.
  (`crates/platform-domain/src/identity.rs`)
- **Knowledge search audience leak (HIGH):** `SearchIndex::query` returned matches regardless
  of `ArticleAudience`, exposing `Internal` runbooks/known-errors to lower-trust callers. Added
  `query_for_audience` with clearance-based filtering; the default `query` is now public-only
  (safe by default). (`crates/search-knowledge/src/lib.rs`)
- **Regulated-data AI egress (HIGH):** `plan_execution` could route regulated data to an
  external API provider, and prompt-retaining providers were never blocked for regulated data.
  Regulated capabilities can no longer target `ExternalApi`, external inference for regulated
  data is always blocked, and prompt retention is rejected for regulated data via the new
  `PromptRetentionNotAllowed` guardrail. (`crates/ai-orchestrator/src/lib.rs`)
- **Frontend stored-XSS sink (MEDIUM-HIGH):** operator views rendered dynamic record fields
  via `innerHTML` without escaping. Added an `escapeHtml` helper applied to all interpolated
  values. (`apps/frontend/app.js`)
- **Missing frontend CSP (MEDIUM):** added a strict same-origin Content-Security-Policy and a
  `no-referrer` policy to the control-plane app. Clickjacking protection is documented as an
  HTTP-header responsibility of the serving layer. (`apps/frontend/index.html`)

### Tests

- Added unit tests covering same-tenant vs cross-tenant authorization, audience-scoped
  knowledge search, regulated external-API blocking, and prompt-retention blocking.

## [0.1.0]

### Added

- Initial Infinity Service Ops platform scaffold: multi-crate Rust workspace, runtime apps,
  shared platform primitives, ITSM/CMDB/ITOM/SecOps domain models, workflow engine, AI
  guardrails, automation library, and the operator-facing frontend control plane.
