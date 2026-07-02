use crate::primitives::{
    AttachmentRef, ExternalRef, Impact, IncidentId, Ownership, Priority, ProblemId, RecordMeta,
    ServiceId, TenantId, Timestamp, Urgency, UserId, derive_priority,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncidentStatus {
    New,
    Triage,
    InProgress,
    Mitigated,
    Resolved,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncidentChannel {
    Portal,
    Email,
    ChatOps,
    Monitoring,
    Api,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incident {
    pub id: IncidentId,
    pub tenant_id: TenantId,
    pub number: String,
    pub title: String,
    pub description: String,
    pub status: IncidentStatus,
    pub impact: Impact,
    pub urgency: Urgency,
    pub priority: Priority,
    pub channel: IncidentChannel,
    pub caller_user_id: Option<UserId>,
    pub affected_service_id: Option<ServiceId>,
    pub ownership: Ownership,
    pub related_problem_id: Option<ProblemId>,
    pub related_change_id: Option<crate::primitives::ChangeRequestId>,
    pub sla_policy_id: Option<String>,
    pub external_refs: Vec<ExternalRef>,
    pub attachments: Vec<AttachmentRef>,
    pub policy_hook_ids: Vec<String>,
    pub meta: RecordMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentWorklog {
    pub id: String,
    pub incident_id: IncidentId,
    pub tenant_id: TenantId,
    pub body: String,
    pub internal: bool,
    pub created_at: Timestamp,
    pub created_by: UserId,
}

impl Incident {
    pub fn recalculate_priority(&mut self) {
        self.priority = derive_priority(&self.impact, &self.urgency);
        self.meta.version += 1;
    }

    pub fn transition(&mut self, status: IncidentStatus) {
        self.status = status;
        self.meta.version += 1;
    }

    pub fn assign(&mut self, assignee_user_id: Option<UserId>) {
        self.ownership.assignee_user_id = assignee_user_id;
        if self.status == IncidentStatus::New {
            self.status = IncidentStatus::Triage;
        }
        self.meta.version += 1;
    }

    pub fn attach_problem(&mut self, problem_id: ProblemId) {
        self.related_problem_id = Some(problem_id);
        self.meta.version += 1;
    }

    pub fn add_worklog(
        &self,
        id: impl Into<String>,
        body: impl Into<String>,
        internal: bool,
        created_by: UserId,
    ) -> IncidentWorklog {
        IncidentWorklog {
            id: id.into(),
            incident_id: self.id.clone(),
            tenant_id: self.tenant_id.clone(),
            body: body.into(),
            internal,
            created_at: self.meta.updated_at.clone(),
            created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Incident, IncidentChannel, IncidentStatus};
    use crate::primitives::{
        ActorRef, Impact, IncidentId, Ownership, Priority, RecordMeta, TeamId, TenantId, Timestamp,
        Urgency, UserId,
    };

    #[test]
    fn assignment_moves_new_incident_to_triage() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("bootstrap");
        let mut incident = Incident {
            id: IncidentId::new("inc-1"),
            tenant_id: tenant_id.clone(),
            number: "INC0001".to_string(),
            title: "Email outage".to_string(),
            description: "Mail is unavailable".to_string(),
            status: IncidentStatus::New,
            impact: Impact::Enterprise,
            urgency: Urgency::High,
            priority: Priority::P2,
            channel: IncidentChannel::Portal,
            caller_user_id: None,
            affected_service_id: None,
            ownership: Ownership::new(TeamId::new("team-a"), None),
            related_problem_id: None,
            related_change_id: None,
            sla_policy_id: None,
            external_refs: vec![],
            attachments: vec![],
            policy_hook_ids: vec![],
            meta: RecordMeta::bootstrap(tenant_id, actor),
        };

        incident.assign(Some(UserId::new("user-a")));
        incident.recalculate_priority();

        assert_eq!(incident.status, IncidentStatus::Triage);
        assert_eq!(incident.priority, Priority::P1);
        assert_eq!(
            incident.meta.updated_at,
            Timestamp::new("1970-01-01T00:00:00Z")
        );
    }
}
