use event_core::EventMetadata;
use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    RecordMutation,
    PolicyDecision,
    ApprovalDecision,
    AiInvocation,
    ServiceAccess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    pub id: String,
    pub tenant_id: TenantId,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub data_classes: Vec<DataClass>,
    pub correlation_id: CorrelationId,
}

impl AuditRecord {
    pub fn from_event(
        id: impl Into<String>,
        action: AuditAction,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        metadata: &EventMetadata,
    ) -> Self {
        Self {
            id: id.into(),
            tenant_id: metadata.tenant_id.clone(),
            action,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            data_classes: vec![metadata.data_class.clone()],
            correlation_id: metadata.correlation_id.clone(),
        }
    }
}

pub fn audit_stream_name(action: AuditAction) -> &'static str {
    match action {
        AuditAction::RecordMutation => "audit.record-mutations",
        AuditAction::PolicyDecision => "audit.policy-decisions",
        AuditAction::ApprovalDecision => "audit.approvals",
        AuditAction::AiInvocation => "audit.ai",
        AuditAction::ServiceAccess => "audit.service-access",
    }
}

pub fn bootstrap_metadata() -> EventMetadata {
    EventMetadata::new(
        "audit.bootstrap",
        ActorId::new("audit-core"),
        TenantId::new("system"),
        CorrelationId::new("audit-bootstrap"),
        DataClass::Internal,
    )
}

#[cfg(test)]
mod tests {
    use super::{AuditAction, AuditRecord, audit_stream_name, bootstrap_metadata};

    #[test]
    fn derives_record_from_event_metadata() {
        let metadata = bootstrap_metadata();
        let record = AuditRecord::from_event(
            "audit-1",
            AuditAction::PolicyDecision,
            "policy_hook",
            "hook-1",
            &metadata,
        );

        assert_eq!(record.tenant_id.0, "system");
        assert_eq!(record.resource_type, "policy_hook");
    }

    #[test]
    fn stream_name_matches_action() {
        assert_eq!(audit_stream_name(AuditAction::AiInvocation), "audit.ai");
    }
}
