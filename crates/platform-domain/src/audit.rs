use crate::primitives::{ActorRef, AuditEventId, DataClass, TenantId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    StateTransition,
    ApprovalDecision,
    PolicyEvaluation,
    DataExport,
    AiAssistance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditResourceRef {
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub tenant_id: TenantId,
    pub occurred_at: Timestamp,
    pub actor: ActorRef,
    pub action: AuditAction,
    pub resource: AuditResourceRef,
    pub data_classes: Vec<DataClass>,
    pub source_ip: Option<String>,
    pub correlation_id: String,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub policy_evaluation_id: Option<String>,
    pub immutable_sequence: u64,
}

impl AuditEvent {
    pub fn for_resource(
        id: AuditEventId,
        tenant_id: TenantId,
        actor: ActorRef,
        action: AuditAction,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        data_classes: Vec<DataClass>,
        correlation_id: impl Into<String>,
        immutable_sequence: u64,
    ) -> Self {
        Self {
            id,
            tenant_id,
            occurred_at: Timestamp::new("1970-01-01T00:00:00Z"),
            actor,
            action,
            resource: AuditResourceRef {
                resource_type: resource_type.into(),
                resource_id: resource_id.into(),
            },
            data_classes,
            source_ip: None,
            correlation_id: correlation_id.into(),
            before_hash: None,
            after_hash: None,
            policy_evaluation_id: None,
            immutable_sequence,
        }
    }

    pub fn touches_resource(&self, resource_type: &str, resource_id: &str) -> bool {
        self.resource.resource_type == resource_type && self.resource.resource_id == resource_id
    }
}
