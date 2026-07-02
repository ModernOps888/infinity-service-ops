pub mod api;
pub mod audit;
pub mod changes;
pub mod database;
pub mod identity;
pub mod incidents;
pub mod knowledge;
pub mod module_map;
pub mod policy_hooks;
pub mod primitives;
pub mod problems;
pub mod service_catalog;
pub mod service_requests;
pub mod sovereignty;
pub mod tenancy;

pub use audit::{AuditAction, AuditEvent};
pub use identity::{AuthorizationContext, Permission, Role, Team, User, UserStatus};
pub use policy_hooks::{PolicyDecision, PolicyEvaluation, PolicyHook, PolicyHookPoint};
pub use primitives::{ActorRef, RecordMeta, TeamId, TenantId, Timestamp, UserId};
pub use sovereignty::{
    Criticality, IntegrationTarget, SovereigntyMode, SovereigntyPolicy, SystemRecord, SystemSource,
};
pub use tenancy::{DataBoundary, Tenant, TenantEnvironment, TenantTier};

#[cfg(test)]
mod tests {
    use crate::{
        ActorRef, AuditAction, AuditEvent, AuthorizationContext, Permission, PolicyDecision,
        PolicyEvaluation, PolicyHookPoint, RecordMeta, Role, Tenant, TenantId, Timestamp, User,
        UserId, UserStatus, api::milestone1_api_resources, database::milestone1_tables,
        module_map::milestone1_module_boundaries,
    };

    #[test]
    fn milestone1_model_covers_core_itsm_domains() {
        let modules = milestone1_module_boundaries();
        for expected in [
            "tenancy",
            "identity",
            "incidents",
            "service_requests",
            "changes",
            "problems",
            "knowledge",
            "service_catalog",
            "audit",
            "policy_hooks",
        ] {
            assert!(
                modules.iter().any(|module| module.module == expected),
                "missing module boundary for {expected}"
            );
        }
    }

    #[test]
    fn milestone1_persistence_and_api_include_policy_and_audit() {
        let tables = milestone1_tables();
        assert!(tables.iter().any(|table| table.name == "audit_events"));
        assert!(tables.iter().any(|table| table.name == "policy_hooks"));
        assert!(tables.iter().any(|table| table.name == "incidents"));
        assert!(tables.iter().any(|table| table.name == "service_requests"));
        assert!(tables.iter().any(|table| table.name == "change_requests"));
        assert!(tables.iter().any(|table| table.name == "problems"));

        let resources = milestone1_api_resources();
        assert!(
            resources
                .iter()
                .any(|resource| resource.path == "/v1/incidents")
        );
        assert!(
            resources
                .iter()
                .any(|resource| resource.path == "/v1/audit/events")
        );
        assert!(
            resources
                .iter()
                .any(|resource| resource.path == "/v1/policy/hooks")
        );
    }

    #[test]
    fn regulated_tenant_only_allows_its_boundary_region() {
        let tenant = Tenant::regulated_default(
            TenantId::new("tenant-a"),
            "tenant-a".to_string(),
            "Tenant A".to_string(),
            "uk-south".to_string(),
        );

        assert!(tenant.is_regulated());
        assert!(tenant.is_operational());
        assert!(tenant.allows_region("uk-south"));
        assert!(!tenant.allows_region("eu-west"));
    }

    #[test]
    fn authorization_requires_active_user_and_granted_role() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("bootstrap");
        let role = Role {
            id: "tenant-admin".to_string(),
            tenant_id: tenant_id.clone(),
            name: "Tenant Admin".to_string(),
            permissions: vec![Permission::ManageTenant, Permission::ReadAudit],
            meta: RecordMeta::bootstrap(tenant_id.clone(), actor),
        };
        let user = User {
            id: UserId::new("user-1"),
            tenant_id,
            external_subject: "sub-1".to_string(),
            email: "admin@example.com".to_string(),
            display_name: "Admin".to_string(),
            status: UserStatus::Active,
            primary_team_id: None,
            role_ids: vec![role.id.clone()],
        };

        let roles = [role];
        let context = AuthorizationContext::new(&user, &roles);
        assert!(context.allows(Permission::ManageTenant));
        assert!(!context.allows(Permission::ApproveChange));
    }

    #[test]
    fn policy_and_audit_helpers_encode_phase_one_controls() {
        let evaluation = PolicyEvaluation {
            id: "eval-1".to_string(),
            tenant_id: TenantId::new("tenant-a"),
            hook_id: crate::primitives::PolicyHookId::new("hook-1"),
            hook_point: PolicyHookPoint::ApprovalDecision,
            resource_type: "change_request".to_string(),
            resource_id: "chg-1".to_string(),
            actor: ActorRef::system("policy-engine"),
            data_classes: vec![crate::primitives::DataClass::Operational],
            decision: PolicyDecision::RequireApproval,
            obligations: vec!["cab-approval".to_string()],
            reason: "Emergency changes require approval".to_string(),
            evaluated_at: Timestamp::new("1970-01-01T00:00:00Z"),
        };

        let event = AuditEvent::for_resource(
            crate::primitives::AuditEventId::new("audit-1"),
            TenantId::new("tenant-a"),
            ActorRef::system("api-gateway"),
            AuditAction::Create,
            "incident",
            "inc-1",
            vec![crate::primitives::DataClass::Operational],
            "corr-1",
            1,
        );

        assert!(evaluation.requires_approval());
        assert!(!evaluation.blocks_operation());
        assert!(event.touches_resource("incident", "inc-1"));
    }
}
