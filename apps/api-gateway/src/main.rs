use audit_core::{AuditAction, audit_stream_name};
use event_core::EventMetadata;
use platform_core::{ActorId, CorrelationId, DataClass, SecurityContext, TenantContext, TenantId};
use platform_domain::{
    ActorRef, AuthorizationContext, Permission, RecordMeta, Role, User, UserId, UserStatus,
};

fn main() {
    let tenant = TenantContext::new(TenantId::new("tenant-demo"), "uk-south");
    let security = SecurityContext::service(
        tenant.id.clone(),
        ActorId::new("api-gateway"),
        vec!["gateway".to_string(), "tenant-router".to_string()],
    );
    let metadata = EventMetadata::new(
        "gateway.bootstrapped",
        ActorId::new("api-gateway"),
        tenant.id.clone(),
        CorrelationId::new("boot-001"),
        DataClass::Operational,
    );
    let audit_role = Role {
        id: "audit-reader".to_string(),
        tenant_id: tenant.id.clone(),
        name: "Audit Reader".to_string(),
        permissions: vec![Permission::ReadAudit],
        meta: RecordMeta::bootstrap(tenant.id.clone(), ActorRef::system("api-gateway")),
    };
    let user = User {
        id: UserId::new("gateway-operator"),
        tenant_id: tenant.id.clone(),
        external_subject: "gateway-operator".to_string(),
        email: "gateway@example.com".to_string(),
        display_name: "Gateway Operator".to_string(),
        status: UserStatus::Active,
        primary_team_id: None,
        role_ids: vec![audit_role.id.clone()],
    };
    let roles = [audit_role];
    let authorization = AuthorizationContext::new(&user, &roles);

    println!("API gateway bootstrap");
    println!("Tenant region: {}", tenant.region);
    println!("Security actor: {}", security.actor_id.0);
    println!("Bootstrap event: {}", metadata.event_type);
    println!(
        "Audit stream: {}",
        audit_stream_name(AuditAction::ServiceAccess)
    );
    println!(
        "Read audit permitted: {}",
        authorization.allows(Permission::ReadAudit)
    );
}
