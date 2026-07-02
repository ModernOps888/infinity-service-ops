use ai_orchestrator::{AiCapability, AiProviderPolicy, DataClass, can_run_capability};
use audit_core::{AuditAction as LedgerAuditAction, AuditRecord};
use automation_library::default_automation_library;
use cmdb_domain::{
    ConfigurationItem, ConfigurationItemClass, ConfigurationItemId, ConfigurationItemStatus,
    Relationship, RelationshipId, RelationshipType, ServiceGraph,
};
use event_core::EventMetadata;
use integration_core::native_connector_catalog;
use platform_core::{ActorId, CorrelationId, DataClass as CoreDataClass};
use platform_domain::{
    ActorRef, AuditAction, AuditEvent, AuthorizationContext, Criticality, Permission,
    PolicyDecision, PolicyEvaluation, PolicyHookPoint, RecordMeta, Role, SovereigntyMode,
    SovereigntyPolicy, SystemRecord, SystemSource, Tenant, TenantId, Timestamp, User, UserId,
    UserStatus,
};
use policy_engine::{DeploymentPlan, evaluate_deployment};
use security_foundation::SecurityBaseline;

fn main() {
    let tenant = Tenant::regulated_default(
        TenantId::new("tenant-uk-1"),
        "tenant-uk-1".to_string(),
        "UK Regulated Tenant".to_string(),
        "uk-south".to_string(),
    );
    let policy = SovereigntyPolicy {
        mode: SovereigntyMode::Hybrid,
        allowed_regions: vec!["eu-west".to_string(), "uk-south".to_string()],
        requires_customer_managed_keys: true,
        allow_external_model_training: false,
    };

    let baseline = SecurityBaseline::zero_trust();
    let connectors = native_connector_catalog();
    let capability = AiCapability {
        name: "incident-triage-copilot",
        requires_human_approval: true,
        data_classes: vec![DataClass::Metadata],
    };

    let ai_policy = AiProviderPolicy::internal_only();
    let ai_result = can_run_capability(&policy, &ai_policy, &capability);
    let control_plane_role = Role {
        id: "tenant-admin".to_string(),
        tenant_id: tenant.id.clone(),
        name: "Tenant Admin".to_string(),
        permissions: vec![Permission::ManageTenant, Permission::ReadAudit],
        meta: RecordMeta::bootstrap(tenant.id.clone(), ActorRef::system("control-plane")),
    };
    let control_plane_user = User {
        id: UserId::new("operator-1"),
        tenant_id: tenant.id.clone(),
        external_subject: "operator-1".to_string(),
        email: "operator@example.com".to_string(),
        display_name: "Platform Operator".to_string(),
        status: UserStatus::Active,
        primary_team_id: None,
        role_ids: vec![control_plane_role.id.clone()],
    };
    let roles = [control_plane_role];
    let authorization = AuthorizationContext::new(&control_plane_user, &roles);

    let plan = DeploymentPlan {
        target_region: "uk-south".to_string(),
        security_baseline: baseline.clone(),
        systems: vec![SystemRecord {
            name: "identity-governance".to_string(),
            owner_team: "platform-security".to_string(),
            source: SystemSource::Microsoft365,
            criticality: Criticality::MissionCritical,
            contains_regulated_data: true,
        }],
    };

    let report = evaluate_deployment(&policy, &plan);
    let policy_evaluation = PolicyEvaluation {
        id: "eval-tenant-boundary".to_string(),
        tenant_id: tenant.id.clone(),
        hook_id: platform_domain::primitives::PolicyHookId::new("hook-tenant-boundary"),
        hook_point: PolicyHookPoint::TenantBoundaryChange,
        resource_type: "tenant".to_string(),
        resource_id: tenant.slug.clone(),
        actor: ActorRef::system("policy-engine"),
        data_classes: vec![platform_domain::primitives::DataClass::Operational],
        decision: PolicyDecision::RequireApproval,
        obligations: vec!["two-person-review".to_string()],
        reason: "Regulated tenant boundary updates require approval".to_string(),
        evaluated_at: Timestamp::new("1970-01-01T00:00:00Z"),
    };
    let event_metadata = EventMetadata::new(
        "tenant.provisioned",
        ActorId::new("control-plane"),
        tenant.id.clone(),
        CorrelationId::new("tenant-provision-001"),
        CoreDataClass::Operational,
    );
    let ledger_record = AuditRecord::from_event(
        "ledger-tenant-provision-001",
        LedgerAuditAction::RecordMutation,
        "tenant",
        tenant.slug.clone(),
        &event_metadata,
    );
    let audit_event = AuditEvent::for_resource(
        platform_domain::primitives::AuditEventId::new("audit-tenant-provision-001"),
        tenant.id.clone(),
        ActorRef::system("control-plane"),
        AuditAction::Create,
        "tenant",
        tenant.slug.clone(),
        vec![platform_domain::primitives::DataClass::Operational],
        "tenant-provision-001",
        1,
    );
    let mut graph = ServiceGraph::default();
    graph.add_item(ConfigurationItem {
        id: ConfigurationItemId::new("svc-identity"),
        tenant_id: tenant.id.clone(),
        name: "Identity Governance".to_string(),
        class: ConfigurationItemClass::BusinessService,
        status: ConfigurationItemStatus::Live,
        criticality: Criticality::MissionCritical,
        owner_team_id: None,
        environment: "prod".to_string(),
        data_classes: vec![platform_core::DataClass::Operational],
        external_refs: vec![],
        tags: vec!["identity".to_string()],
    });
    graph.add_item(ConfigurationItem {
        id: ConfigurationItemId::new("app-entra-sync"),
        tenant_id: tenant.id.clone(),
        name: "Entra Sync".to_string(),
        class: ConfigurationItemClass::Application,
        status: ConfigurationItemStatus::Live,
        criticality: Criticality::MissionCritical,
        owner_team_id: None,
        environment: "prod".to_string(),
        data_classes: vec![platform_core::DataClass::Operational],
        external_refs: vec![],
        tags: vec!["connector".to_string()],
    });
    graph.add_relationship(Relationship {
        id: RelationshipId::new("rel-identity-1"),
        tenant_id: tenant.id.clone(),
        from: ConfigurationItemId::new("svc-identity"),
        to: ConfigurationItemId::new("app-entra-sync"),
        relationship_type: RelationshipType::DependsOn,
        discovered: true,
    });
    let impacted_items = graph.downstream_of(&ConfigurationItemId::new("svc-identity"));
    let automation_library = default_automation_library();
    let enterprise_templates = automation_library
        .iter()
        .filter(|template| template.is_enterprise_ready())
        .count();

    println!("Infinity Service Ops bootstrap");
    println!(
        "Tenant {} operational in region uk-south: {}",
        tenant.display_name,
        tenant.allows_region("uk-south")
    );
    println!("Connectors available: {}", connectors.len());
    println!(
        "AI capability status: {}",
        if ai_result.is_ok() {
            "approved"
        } else {
            "blocked"
        }
    );
    println!(
        "Deployment posture: {}",
        if report.is_approved() {
            "approved"
        } else {
            "needs remediation"
        }
    );
    println!(
        "Tenant management permitted: {}",
        authorization.allows(Permission::ManageTenant)
    );
    println!(
        "Boundary approval required: {}",
        policy_evaluation.requires_approval()
    );
    println!("Ledger audit action: {:?}", ledger_record.action);
    println!(
        "Domain audit targets tenant: {}",
        audit_event.touches_resource("tenant", &tenant.slug)
    );
    println!("CMDB downstream dependencies: {}", impacted_items.len());
    println!("Automation templates: {}", automation_library.len());
    println!("Enterprise-ready templates: {}", enterprise_templates);
}
