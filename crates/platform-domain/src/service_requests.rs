use crate::primitives::{
    CatalogItemId, Ownership, RecordMeta, ServiceRequestId, TenantId, Timestamp, UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceRequestStatus {
    Draft,
    Submitted,
    WaitingApproval,
    Approved,
    InFulfillment,
    Completed,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalDecision {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestApproval {
    pub id: String,
    pub tenant_id: TenantId,
    pub request_id: ServiceRequestId,
    pub approver_user_id: UserId,
    pub decision: ApprovalDecision,
    pub decided_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestVariableValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequest {
    pub id: ServiceRequestId,
    pub tenant_id: TenantId,
    pub number: String,
    pub catalog_item_id: CatalogItemId,
    pub requester_user_id: UserId,
    pub requested_for_user_id: Option<UserId>,
    pub status: ServiceRequestStatus,
    pub ownership: Ownership,
    pub variables: Vec<RequestVariableValue>,
    pub approvals: Vec<RequestApproval>,
    pub due_at: Option<Timestamp>,
    pub policy_hook_ids: Vec<String>,
    pub meta: RecordMeta,
}

impl RequestApproval {
    pub fn record_decision(&mut self, decision: ApprovalDecision, decided_at: Timestamp) {
        self.decision = decision;
        self.decided_at = Some(decided_at);
    }
}

impl ServiceRequest {
    pub fn submit(&mut self) {
        self.status = if self.approvals.is_empty() {
            ServiceRequestStatus::Submitted
        } else {
            ServiceRequestStatus::WaitingApproval
        };
        self.meta.version += 1;
    }

    pub fn record_approval(
        &mut self,
        approval_id: &str,
        decision: ApprovalDecision,
        decided_at: Timestamp,
    ) -> bool {
        let mut updated = false;
        for approval in &mut self.approvals {
            if approval.id == approval_id {
                approval.record_decision(decision.clone(), decided_at.clone());
                updated = true;
            }
        }

        if updated {
            if self
                .approvals
                .iter()
                .any(|approval| approval.decision == ApprovalDecision::Rejected)
            {
                self.status = ServiceRequestStatus::Rejected;
            } else if self
                .approvals
                .iter()
                .all(|approval| approval.decision == ApprovalDecision::Approved)
            {
                self.status = ServiceRequestStatus::Approved;
            }
            self.meta.version += 1;
        }

        updated
    }

    pub fn start_fulfillment(&mut self) -> bool {
        let allowed = matches!(
            self.status,
            ServiceRequestStatus::Submitted | ServiceRequestStatus::Approved
        );
        if allowed {
            self.status = ServiceRequestStatus::InFulfillment;
            self.meta.version += 1;
        }
        allowed
    }

    pub fn complete(&mut self) -> bool {
        if self.status == ServiceRequestStatus::InFulfillment {
            self.status = ServiceRequestStatus::Completed;
            self.meta.version += 1;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::{ApprovalDecision, RequestApproval, ServiceRequest, ServiceRequestStatus};
    use crate::primitives::{
        ActorRef, CatalogItemId, Ownership, RecordMeta, TeamId, TenantId, Timestamp, UserId,
    };

    #[test]
    fn approvals_drive_request_into_fulfillment() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("bootstrap");
        let mut request = ServiceRequest {
            id: crate::primitives::ServiceRequestId::new("req-1"),
            tenant_id: tenant_id.clone(),
            number: "REQ0001".to_string(),
            catalog_item_id: CatalogItemId::new("cat-1"),
            requester_user_id: UserId::new("requester"),
            requested_for_user_id: None,
            status: ServiceRequestStatus::Draft,
            ownership: Ownership::new(TeamId::new("team-a"), None),
            variables: vec![],
            approvals: vec![RequestApproval {
                id: "approval-1".to_string(),
                tenant_id: tenant_id.clone(),
                request_id: crate::primitives::ServiceRequestId::new("req-1"),
                approver_user_id: UserId::new("approver"),
                decision: ApprovalDecision::Pending,
                decided_at: None,
            }],
            due_at: None,
            policy_hook_ids: vec![],
            meta: RecordMeta::bootstrap(tenant_id, actor),
        };

        request.submit();
        assert_eq!(request.status, ServiceRequestStatus::WaitingApproval);

        assert!(request.record_approval(
            "approval-1",
            ApprovalDecision::Approved,
            Timestamp::new("1970-01-01T00:00:00Z")
        ));
        assert_eq!(request.status, ServiceRequestStatus::Approved);
        assert!(request.start_fulfillment());
        assert!(request.complete());
        assert_eq!(request.status, ServiceRequestStatus::Completed);
    }
}
