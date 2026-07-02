use crate::primitives::{ActorRef, DataClass, PolicyHookId, TenantId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyHookPoint {
    TicketCreate,
    TicketTransition,
    AssignmentChange,
    ApprovalDecision,
    ChangeSchedule,
    CatalogFulfillment,
    KnowledgePublish,
    DataExport,
    AiAssist,
    TenantBoundaryChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyEnforcementMode {
    Advisory,
    Blocking,
    ApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    RequireApproval,
    RequireRedaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyHook {
    pub id: PolicyHookId,
    pub tenant_id: TenantId,
    pub name: String,
    pub hook_point: PolicyHookPoint,
    pub enforcement: PolicyEnforcementMode,
    pub applies_to_resource_types: Vec<String>,
    pub expression_ref: String,
    pub active: bool,
}

impl PolicyHook {
    pub fn applies_to(&self, resource_type: &str) -> bool {
        self.applies_to_resource_types
            .iter()
            .any(|current| current == resource_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEvaluation {
    pub id: String,
    pub tenant_id: TenantId,
    pub hook_id: PolicyHookId,
    pub hook_point: PolicyHookPoint,
    pub resource_type: String,
    pub resource_id: String,
    pub actor: ActorRef,
    pub data_classes: Vec<DataClass>,
    pub decision: PolicyDecision,
    pub obligations: Vec<String>,
    pub reason: String,
    pub evaluated_at: Timestamp,
}

impl PolicyEvaluation {
    pub fn blocks_operation(&self) -> bool {
        self.decision == PolicyDecision::Deny
    }

    pub fn requires_approval(&self) -> bool {
        self.decision == PolicyDecision::RequireApproval
    }
}
