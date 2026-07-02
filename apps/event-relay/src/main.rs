use audit_core::{AuditAction, AuditRecord};
use event_core::EventMetadata;
use persistence_core::{InboxMessage, OutboxMessage};
use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

fn main() {
    let metadata = EventMetadata::new(
        "incident.created",
        ActorId::new("event-relay"),
        TenantId::new("tenant-demo"),
        CorrelationId::new("relay-001"),
        DataClass::Operational,
    );
    let outbox = OutboxMessage::new("outbox-1", TenantId::new("tenant-demo"), metadata.clone());
    let inbox = InboxMessage::new("inbox-1", TenantId::new("tenant-demo"), metadata.clone());
    let audit = AuditRecord::from_event(
        "audit-1",
        AuditAction::RecordMutation,
        "outbox_message",
        outbox.id.clone(),
        &metadata,
    );

    println!("Event relay bootstrap");
    println!("Outbox event: {}", outbox.event.event_type);
    println!("Inbox processed: {}", inbox.processed);
    println!("Audit resource: {}", audit.resource_type);
}
