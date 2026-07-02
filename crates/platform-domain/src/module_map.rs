#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleBoundary {
    pub module: &'static str,
    pub crate_path: &'static str,
    pub owns: &'static [&'static str],
    pub depends_on: &'static [&'static str],
    pub exposes: &'static [&'static str],
}

pub fn milestone1_module_boundaries() -> Vec<ModuleBoundary> {
    vec![
        ModuleBoundary {
            module: "primitives",
            crate_path: "platform_domain::primitives",
            owns: &[
                "typed IDs",
                "timestamps",
                "priority/impact/urgency",
                "record metadata",
                "actor references",
            ],
            depends_on: &[],
            exposes: &[
                "newtype identifiers",
                "RecordMeta",
                "Ownership",
                "DataClass",
            ],
        },
        ModuleBoundary {
            module: "tenancy",
            crate_path: "platform_domain::tenancy",
            owns: &[
                "tenant",
                "tenant environment",
                "data boundary",
                "sovereign defaults",
            ],
            depends_on: &["primitives", "sovereignty"],
            exposes: &["Tenant", "TenantEnvironment", "DataBoundary"],
        },
        ModuleBoundary {
            module: "identity",
            crate_path: "platform_domain::identity",
            owns: &["users", "teams", "roles", "team memberships"],
            depends_on: &["primitives", "tenancy"],
            exposes: &["User", "Team", "Role", "Permission"],
        },
        ModuleBoundary {
            module: "service_catalog",
            crate_path: "platform_domain::service_catalog",
            owns: &[
                "services",
                "catalog categories",
                "catalog items",
                "intake variables",
                "fulfillment references",
            ],
            depends_on: &["primitives", "identity", "sovereignty"],
            exposes: &[
                "ServiceOffering",
                "ServiceCatalogItem",
                "ServiceCatalogCategory",
            ],
        },
        ModuleBoundary {
            module: "incidents",
            crate_path: "platform_domain::incidents",
            owns: &[
                "incident aggregate",
                "worklogs",
                "incident priority",
                "incident links",
            ],
            depends_on: &[
                "primitives",
                "identity",
                "service_catalog",
                "problems",
                "changes",
            ],
            exposes: &["Incident", "IncidentWorklog", "IncidentStatus"],
        },
        ModuleBoundary {
            module: "service_requests",
            crate_path: "platform_domain::service_requests",
            owns: &[
                "request aggregate",
                "catalog variable values",
                "request approvals",
            ],
            depends_on: &["primitives", "identity", "service_catalog", "policy_hooks"],
            exposes: &["ServiceRequest", "RequestApproval"],
        },
        ModuleBoundary {
            module: "changes",
            crate_path: "platform_domain::changes",
            owns: &[
                "change aggregate",
                "change tasks",
                "CAB approvals",
                "planned windows",
                "risk",
            ],
            depends_on: &["primitives", "identity", "service_catalog", "policy_hooks"],
            exposes: &["ChangeRequest", "ChangeTask", "ChangeApproval"],
        },
        ModuleBoundary {
            module: "problems",
            crate_path: "platform_domain::problems",
            owns: &[
                "problem aggregate",
                "known errors",
                "incident-to-problem links",
                "root-cause fields",
            ],
            depends_on: &["primitives", "incidents", "knowledge"],
            exposes: &["Problem", "KnownError"],
        },
        ModuleBoundary {
            module: "knowledge",
            crate_path: "platform_domain::knowledge",
            owns: &[
                "knowledge articles",
                "runbooks",
                "feedback",
                "review lifecycle",
            ],
            depends_on: &["primitives", "identity", "problems", "policy_hooks"],
            exposes: &["KnowledgeArticle", "KnowledgeFeedback"],
        },
        ModuleBoundary {
            module: "policy_hooks",
            crate_path: "platform_domain::policy_hooks",
            owns: &[
                "hook points",
                "enforcement modes",
                "policy decisions",
                "policy evaluation ledger",
            ],
            depends_on: &["primitives", "tenancy", "identity"],
            exposes: &["PolicyHook", "PolicyEvaluation", "PolicyHookPoint"],
        },
        ModuleBoundary {
            module: "audit",
            crate_path: "platform_domain::audit",
            owns: &[
                "append-only event envelope",
                "resource references",
                "correlation and hashes",
            ],
            depends_on: &["primitives", "policy_hooks"],
            exposes: &["AuditEvent", "AuditAction"],
        },
        ModuleBoundary {
            module: "database",
            crate_path: "platform_domain::database",
            owns: &[
                "Milestone 1 logical tables",
                "tenant scoping",
                "index recommendations",
            ],
            depends_on: &["all domain modules"],
            exposes: &["milestone1_tables"],
        },
        ModuleBoundary {
            module: "api",
            crate_path: "platform_domain::api",
            owns: &[
                "Milestone 1 REST resources",
                "allowed methods",
                "audit and policy attachment metadata",
            ],
            depends_on: &["database", "policy_hooks", "audit"],
            exposes: &["milestone1_api_resources"],
        },
    ]
}
