use crate::primitives::{IncidentId, Ownership, ProblemId, RecordMeta, TenantId, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemStatus {
    New,
    Investigating,
    KnownError,
    FixScheduled,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownError {
    pub id: String,
    pub tenant_id: TenantId,
    pub problem_id: ProblemId,
    pub symptom: String,
    pub workaround: String,
    pub published_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub id: ProblemId,
    pub tenant_id: TenantId,
    pub number: String,
    pub title: String,
    pub description: String,
    pub status: ProblemStatus,
    pub ownership: Ownership,
    pub linked_incident_ids: Vec<IncidentId>,
    pub root_cause: Option<String>,
    pub workaround: Option<String>,
    pub known_error_id: Option<String>,
    pub meta: RecordMeta,
}

impl Problem {
    pub fn link_incident(&mut self, incident_id: IncidentId) {
        if !self.linked_incident_ids.contains(&incident_id) {
            self.linked_incident_ids.push(incident_id);
            if self.status == ProblemStatus::New {
                self.status = ProblemStatus::Investigating;
            }
            self.meta.version += 1;
        }
    }

    pub fn publish_known_error(
        &mut self,
        known_error_id: impl Into<String>,
        workaround: impl Into<String>,
    ) {
        self.known_error_id = Some(known_error_id.into());
        self.workaround = Some(workaround.into());
        self.status = ProblemStatus::KnownError;
        self.meta.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{Problem, ProblemStatus};
    use crate::primitives::{
        ActorRef, IncidentId, Ownership, ProblemId, RecordMeta, TeamId, TenantId,
    };

    #[test]
    fn linking_incident_and_publishing_known_error_advances_problem() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("bootstrap");
        let mut problem = Problem {
            id: ProblemId::new("prb-1"),
            tenant_id: tenant_id.clone(),
            number: "PRB0001".to_string(),
            title: "Recurring mail outage".to_string(),
            description: "Mail repeatedly fails after patching".to_string(),
            status: ProblemStatus::New,
            ownership: Ownership::new(TeamId::new("team-a"), None),
            linked_incident_ids: vec![],
            root_cause: None,
            workaround: None,
            known_error_id: None,
            meta: RecordMeta::bootstrap(tenant_id, actor),
        };

        problem.link_incident(IncidentId::new("inc-1"));
        problem.publish_known_error("ke-1", "Restart the mail transport service");

        assert_eq!(problem.status, ProblemStatus::KnownError);
        assert_eq!(problem.linked_incident_ids.len(), 1);
        assert_eq!(problem.known_error_id.as_deref(), Some("ke-1"));
    }
}
