# Changelog

All notable changes to Infinity Service Ops are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-07-04

### Security

Runtime workflow guardrail enforcement — the roadmap item "enforce workflow guardrails at
runtime" is now implemented, closing several approval-gate bypass vulnerabilities:

- **Approval-gate bypass (HIGH):** `WorkflowExecution::approve` accepted *any* approval id
  (including nonexistent gates) and immediately unlocked execution, never validated the
  approver's role, and a single approval satisfied multiple gates. Approvals now require an
  existing gate, a role match, and **all** gates approved before the execution runs.
  (`crates/workflow-engine/src/lib.rs`)
- **Missing separation of duties (HIGH):** the actor that started an execution could approve
  it. Self-approval is now rejected with `GuardrailViolation::SelfApprovalNotAllowed`.
- **Step execution before approval (HIGH):** `mark_step_complete` and `retry_step` operated
  while the execution was `WaitingApproval` — `retry_step` even force-flipped the status to
  `Running`, a one-call approval bypass. Both now reject non-`Running` executions.
- **Approval replay (MEDIUM):** a gate could be "approved" repeatedly; replays are now
  rejected and each gate records its approver and timestamp (`approved_by`).
- **Unbounded retries (MEDIUM):** steps could be retried forever. A per-step
  `max_step_attempts` budget (default 3) is enforced; exhaustion fails the execution.

### Added

- Typed `GuardrailViolation` error enum so every guardrail failure is auditable rather than
  a silent no-op.
- CI hardening: least-privilege workflow token (`contents: read`),
  `cargo clippy --workspace --all-targets -- -D warnings`, and a RustSec `cargo audit`
  supply-chain job. (`.github/workflows/ci.yml`)
- Security-driven milestone roadmap in the README.

### Tests

- Ten workflow-engine tests covering approval blocking, unknown-gate rejection, role
  mismatch, self-approval rejection, multi-gate completion, replay rejection, retry/step
  approval bypass attempts, and retry-budget exhaustion.

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
