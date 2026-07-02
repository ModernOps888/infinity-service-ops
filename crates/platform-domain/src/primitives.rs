pub use platform_core::{
    ActorRef, AttachmentRef, AuditEventId, CatalogItemId, ChangeRequestId, DataClass, ExternalRef,
    Impact, IncidentId, KnowledgeArticleId, Ownership, PolicyHookId, Priority, ProblemId,
    RecordMeta, RecordStatus, ServiceId, ServiceRequestId, TeamId, TenantId, Timestamp, Urgency,
    UserId,
};

pub fn derive_priority(impact: &Impact, urgency: &Urgency) -> Priority {
    match (impact, urgency) {
        (Impact::Enterprise, Urgency::Critical | Urgency::High) => Priority::P1,
        (Impact::Department, Urgency::Critical) => Priority::P1,
        (Impact::Enterprise | Impact::Department, Urgency::Medium)
        | (Impact::Team, Urgency::Critical | Urgency::High) => Priority::P2,
        (Impact::Individual, Urgency::Critical | Urgency::High)
        | (Impact::Team, Urgency::Medium)
        | (_, Urgency::Low) => Priority::P3,
        _ => Priority::P4,
    }
}
