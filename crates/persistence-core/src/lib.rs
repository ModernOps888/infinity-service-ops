use audit_core::AuditRecord;
use event_core::EventMetadata;
use platform_core::TenantId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxMessage {
    pub id: String,
    pub tenant_id: TenantId,
    pub event: EventMetadata,
    pub attempts: u32,
}

impl OutboxMessage {
    pub fn new(id: impl Into<String>, tenant_id: TenantId, event: EventMetadata) -> Self {
        Self {
            id: id.into(),
            tenant_id,
            event,
            attempts: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxMessage {
    pub id: String,
    pub tenant_id: TenantId,
    pub event: EventMetadata,
    pub processed: bool,
}

impl InboxMessage {
    pub fn new(id: impl Into<String>, tenant_id: TenantId, event: EventMetadata) -> Self {
        Self {
            id: id.into(),
            tenant_id,
            event,
            processed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    Conflict(String),
    NotFound(String),
    Backend(String),
}

pub trait TransactionBoundary {
    fn commit(self) -> Result<(), PersistenceError>;
}

pub trait OutboxStore {
    fn append(&mut self, message: OutboxMessage) -> Result<(), PersistenceError>;
}

pub trait InboxStore {
    fn record(&mut self, message: InboxMessage) -> Result<(), PersistenceError>;
}

pub trait AuditStore {
    fn append_audit(&mut self, record: AuditRecord) -> Result<(), PersistenceError>;
}

pub trait TenantScopedRepository<T> {
    fn list_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<T>, PersistenceError>;
}

#[cfg(test)]
mod tests {
    use super::{InboxMessage, OutboxMessage};
    use event_core::EventMetadata;
    use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

    #[test]
    fn initializes_outbox_message_with_zero_attempts() {
        let event = EventMetadata::new(
            "incident.created",
            ActorId::new("api"),
            TenantId::new("tenant-a"),
            CorrelationId::new("corr-1"),
            DataClass::Operational,
        );

        let message = OutboxMessage::new("outbox-1", TenantId::new("tenant-a"), event);
        assert_eq!(message.attempts, 0);
    }

    #[test]
    fn initializes_inbox_message_as_unprocessed() {
        let event = EventMetadata::new(
            "incident.created",
            ActorId::new("worker"),
            TenantId::new("tenant-a"),
            CorrelationId::new("corr-2"),
            DataClass::Operational,
        );

        let message = InboxMessage::new("inbox-1", TenantId::new("tenant-a"), event);
        assert!(!message.processed);
    }
}
