#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TeamId(pub String);

impl TeamId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(pub String);

impl ServiceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatalogItemId(pub String);

impl CatalogItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncidentId(pub String);

impl IncidentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceRequestId(pub String);

impl ServiceRequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChangeRequestId(pub String);

impl ChangeRequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemId(pub String);

impl ProblemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KnowledgeArticleId(pub String);

impl KnowledgeArticleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolicyHookId(pub String);

impl PolicyHookId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuditEventId(pub String);

impl AuditEventId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(pub String);

impl ActorId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Timestamp(pub String);

impl Timestamp {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataClass {
    Public,
    Internal,
    Operational,
    Regulated,
    Secret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordStatus {
    Draft,
    Active,
    Suspended,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Impact {
    Enterprise,
    Department,
    Team,
    Individual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Urgency {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorRef {
    pub user_id: Option<UserId>,
    pub service_account_id: Option<String>,
    pub display_name: String,
}

impl ActorRef {
    pub fn system(display_name: impl Into<String>) -> Self {
        Self {
            user_id: None,
            service_account_id: None,
            display_name: display_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ownership {
    pub owning_team_id: TeamId,
    pub assignee_user_id: Option<UserId>,
}

impl Ownership {
    pub fn new(owning_team_id: TeamId, assignee_user_id: Option<UserId>) -> Self {
        Self {
            owning_team_id,
            assignee_user_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordMeta {
    pub tenant_id: TenantId,
    pub created_at: Timestamp,
    pub created_by: ActorRef,
    pub updated_at: Timestamp,
    pub updated_by: ActorRef,
    pub version: u64,
}

impl RecordMeta {
    pub fn bootstrap(tenant_id: TenantId, actor: ActorRef) -> Self {
        Self {
            tenant_id,
            created_at: Timestamp::new("1970-01-01T00:00:00Z"),
            created_by: actor.clone(),
            updated_at: Timestamp::new("1970-01-01T00:00:00Z"),
            updated_by: actor,
            version: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentRef {
    pub id: String,
    pub storage_ref: String,
    pub file_name: String,
    pub content_type: String,
    pub data_class: DataClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRef {
    pub system: String,
    pub external_id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantContext {
    pub id: TenantId,
    pub region: String,
}

impl TenantContext {
    pub fn new(id: TenantId, region: impl Into<String>) -> Self {
        Self {
            id,
            region: region.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityContext {
    pub tenant_id: TenantId,
    pub actor_id: ActorId,
    pub roles: Vec<String>,
}

impl SecurityContext {
    pub fn service(tenant_id: TenantId, actor_id: ActorId, roles: Vec<String>) -> Self {
        Self {
            tenant_id,
            actor_id,
            roles,
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|current| current == role)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRequest {
    pub limit: usize,
    pub offset: usize,
}

impl PageRequest {
    pub fn bounded(limit: usize, offset: usize) -> Self {
        Self {
            limit: limit.min(500),
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ActorId, ActorRef, PageRequest, RecordMeta, SecurityContext, TenantId, Timestamp};

    #[test]
    fn limits_page_size_to_safe_bound() {
        let page = PageRequest::bounded(5_000, 0);
        assert_eq!(page.limit, 500);
    }

    #[test]
    fn service_context_checks_roles() {
        let context = SecurityContext::service(
            TenantId::new("tenant-a"),
            ActorId::new("worker"),
            vec!["worker".to_string(), "outbox-dispatcher".to_string()],
        );

        assert!(context.has_role("worker"));
        assert!(!context.has_role("admin"));
    }

    #[test]
    fn bootstrap_meta_sets_initial_version_and_actor() {
        let actor = ActorRef::system("control-plane");
        let meta = RecordMeta::bootstrap(TenantId::new("tenant-a"), actor.clone());

        assert_eq!(meta.version, 1);
        assert_eq!(meta.created_by, actor);
        assert_eq!(meta.created_at, Timestamp::new("1970-01-01T00:00:00Z"));
    }
}
