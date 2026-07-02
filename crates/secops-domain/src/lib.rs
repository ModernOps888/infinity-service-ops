use cmdb_domain::ConfigurationItemId;
use itom_domain::EventSeverity;
use platform_core::{ActorRef, TenantId, Timestamp, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingStatus {
    New,
    Triaged,
    Remediating,
    Resolved,
    FalsePositive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseStatus {
    New,
    Investigating,
    Contained,
    Remediating,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityFinding {
    pub id: String,
    pub tenant_id: TenantId,
    pub title: String,
    pub severity: EventSeverity,
    pub affected_ci_id: Option<ConfigurationItemId>,
    pub source_event_id: Option<String>,
    pub status: FindingStatus,
    pub assignee_user_id: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub id: String,
    pub storage_ref: String,
    pub collected_at: Timestamp,
    pub collected_by: ActorRef,
    pub chain_of_custody_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemediationTask {
    pub id: String,
    pub title: String,
    pub owner_user_id: Option<UserId>,
    pub completed_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityCase {
    pub id: String,
    pub tenant_id: TenantId,
    pub title: String,
    pub status: CaseStatus,
    pub finding_ids: Vec<String>,
    pub evidence: Vec<EvidenceItem>,
    pub remediation_tasks: Vec<RemediationTask>,
}

impl SecurityFinding {
    pub fn triage(&mut self, assignee_user_id: Option<UserId>) {
        self.status = FindingStatus::Triaged;
        self.assignee_user_id = assignee_user_id;
    }

    pub fn start_remediation(&mut self) {
        self.status = FindingStatus::Remediating;
    }

    pub fn resolve(&mut self) {
        self.status = FindingStatus::Resolved;
    }
}

impl SecurityCase {
    pub fn promote_from_finding(id: impl Into<String>, finding: &SecurityFinding) -> Self {
        Self {
            id: id.into(),
            tenant_id: finding.tenant_id.clone(),
            title: finding.title.clone(),
            status: CaseStatus::New,
            finding_ids: vec![finding.id.clone()],
            evidence: vec![],
            remediation_tasks: vec![],
        }
    }

    pub fn add_evidence(&mut self, evidence: EvidenceItem) {
        self.evidence.push(evidence);
        if self.status == CaseStatus::New {
            self.status = CaseStatus::Investigating;
        }
    }

    pub fn add_remediation_task(&mut self, task: RemediationTask) {
        self.remediation_tasks.push(task);
        self.status = CaseStatus::Remediating;
    }

    pub fn close_if_complete(&mut self) -> bool {
        let ready = self
            .remediation_tasks
            .iter()
            .all(|task| task.completed_at.is_some());
        if ready && !self.remediation_tasks.is_empty() {
            self.status = CaseStatus::Resolved;
        }
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CaseStatus, EvidenceItem, FindingStatus, RemediationTask, SecurityCase, SecurityFinding,
    };
    use cmdb_domain::ConfigurationItemId;
    use itom_domain::EventSeverity;
    use platform_core::{ActorRef, TenantId, Timestamp, UserId};

    #[test]
    fn finding_promotes_into_case_and_tracks_evidence() {
        let mut finding = SecurityFinding {
            id: "finding-1".to_string(),
            tenant_id: TenantId::new("tenant-a"),
            title: "Critical CVE on mail node".to_string(),
            severity: EventSeverity::Critical,
            affected_ci_id: Some(ConfigurationItemId::new("srv-mail-1")),
            source_event_id: Some("evt-1".to_string()),
            status: FindingStatus::New,
            assignee_user_id: None,
        };
        finding.triage(Some(UserId::new("sec-user")));

        let mut case = SecurityCase::promote_from_finding("case-1", &finding);
        case.add_evidence(EvidenceItem {
            id: "evidence-1".to_string(),
            storage_ref: "object://evidence-1".to_string(),
            collected_at: Timestamp::new("1970-01-01T00:00:00Z"),
            collected_by: ActorRef::system("secops"),
            chain_of_custody_note: "Collected from scanner export".to_string(),
        });

        assert_eq!(finding.status, FindingStatus::Triaged);
        assert_eq!(case.status, CaseStatus::Investigating);
        assert_eq!(case.evidence.len(), 1);
    }

    #[test]
    fn case_closes_after_all_remediations_complete() {
        let finding = SecurityFinding {
            id: "finding-1".to_string(),
            tenant_id: TenantId::new("tenant-a"),
            title: "Critical CVE on mail node".to_string(),
            severity: EventSeverity::Critical,
            affected_ci_id: None,
            source_event_id: None,
            status: FindingStatus::New,
            assignee_user_id: None,
        };
        let mut case = SecurityCase::promote_from_finding("case-1", &finding);
        case.add_remediation_task(RemediationTask {
            id: "task-1".to_string(),
            title: "Patch node".to_string(),
            owner_user_id: Some(UserId::new("ops-user")),
            completed_at: Some(Timestamp::new("1970-01-01T01:00:00Z")),
        });

        assert!(case.close_if_complete());
        assert_eq!(case.status, CaseStatus::Resolved);
    }
}
