use crate::primitives::{
    ChangeRequestId, Ownership, RecordMeta, ServiceId, TenantId, Timestamp, UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Standard,
    Normal,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeStatus {
    Draft,
    Assess,
    Authorize,
    Scheduled,
    Implementing,
    Review,
    Successful,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeApproval {
    pub id: String,
    pub tenant_id: TenantId,
    pub change_id: ChangeRequestId,
    pub approver_user_id: UserId,
    pub approved: Option<bool>,
    pub decided_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeTask {
    pub id: String,
    pub tenant_id: TenantId,
    pub change_id: ChangeRequestId,
    pub title: String,
    pub ownership: Ownership,
    pub completed_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRequest {
    pub id: ChangeRequestId,
    pub tenant_id: TenantId,
    pub number: String,
    pub title: String,
    pub description: String,
    pub change_type: ChangeType,
    pub risk: ChangeRisk,
    pub status: ChangeStatus,
    pub affected_service_ids: Vec<ServiceId>,
    pub ownership: Ownership,
    pub planned_start: Timestamp,
    pub planned_end: Timestamp,
    pub implementation_plan: String,
    pub rollback_plan: String,
    pub test_plan: String,
    pub approvals: Vec<ChangeApproval>,
    pub policy_hook_ids: Vec<String>,
    pub meta: RecordMeta,
}

impl ChangeRequest {
    pub fn needs_explicit_approval(&self) -> bool {
        !matches!(self.change_type, ChangeType::Standard)
            || matches!(self.risk, ChangeRisk::High | ChangeRisk::Critical)
    }

    pub fn record_approval(
        &mut self,
        approval_id: &str,
        approved: bool,
        decided_at: Timestamp,
    ) -> bool {
        let mut updated = false;
        for approval in &mut self.approvals {
            if approval.id == approval_id {
                approval.approved = Some(approved);
                approval.decided_at = Some(decided_at.clone());
                updated = true;
            }
        }

        if updated {
            self.meta.version += 1;
        }

        updated
    }

    pub fn can_schedule(&self) -> bool {
        !self.needs_explicit_approval()
            || self
                .approvals
                .iter()
                .all(|approval| approval.approved == Some(true))
    }

    pub fn schedule(&mut self) -> bool {
        if self.can_schedule() {
            self.status = ChangeStatus::Scheduled;
            self.meta.version += 1;
            return true;
        }

        false
    }

    pub fn start_implementation(&mut self) -> bool {
        if self.status == ChangeStatus::Scheduled {
            self.status = ChangeStatus::Implementing;
            self.meta.version += 1;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::{ChangeApproval, ChangeRequest, ChangeRisk, ChangeStatus, ChangeType};
    use crate::primitives::{
        ActorRef, ChangeRequestId, Ownership, RecordMeta, ServiceId, TeamId, TenantId, Timestamp,
        UserId,
    };

    #[test]
    fn high_risk_change_requires_approval_before_scheduling() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("bootstrap");
        let mut change = ChangeRequest {
            id: ChangeRequestId::new("chg-1"),
            tenant_id: tenant_id.clone(),
            number: "CHG0001".to_string(),
            title: "Rotate gateway certs".to_string(),
            description: "Rotate production certificates".to_string(),
            change_type: ChangeType::Normal,
            risk: ChangeRisk::High,
            status: ChangeStatus::Authorize,
            affected_service_ids: vec![ServiceId::new("svc-1")],
            ownership: Ownership::new(TeamId::new("team-a"), None),
            planned_start: Timestamp::new("1970-01-01T00:00:00Z"),
            planned_end: Timestamp::new("1970-01-01T01:00:00Z"),
            implementation_plan: "Apply rollout".to_string(),
            rollback_plan: "Restore previous cert".to_string(),
            test_plan: "Validate gateway health".to_string(),
            approvals: vec![ChangeApproval {
                id: "approval-1".to_string(),
                tenant_id: tenant_id.clone(),
                change_id: ChangeRequestId::new("chg-1"),
                approver_user_id: UserId::new("cab-user"),
                approved: None,
                decided_at: None,
            }],
            policy_hook_ids: vec![],
            meta: RecordMeta::bootstrap(tenant_id, actor),
        };

        assert!(change.needs_explicit_approval());
        assert!(!change.schedule());
        assert!(change.record_approval("approval-1", true, Timestamp::new("1970-01-01T00:00:00Z")));
        assert!(change.schedule());
        assert!(change.start_implementation());
        assert_eq!(change.status, ChangeStatus::Implementing);
    }
}
