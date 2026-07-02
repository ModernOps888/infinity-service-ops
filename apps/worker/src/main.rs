use event_core::EventMetadata;
use persistence_core::{InboxMessage, OutboxMessage};
use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

fn main() {
    let tenant_id = TenantId::new("tenant-demo");
    let event = EventMetadata::new(
        "worker.bootstrapped",
        ActorId::new("worker"),
        tenant_id.clone(),
        CorrelationId::new("worker-boot-001"),
        DataClass::Operational,
    );

    let outbox = OutboxMessage::new("outbox-1", tenant_id.clone(), event.clone());
    let inbox = InboxMessage::new("inbox-1", tenant_id, event);

    println!("Worker bootstrap");
    println!("Outbox message: {}", outbox.id);
    println!("Inbox message: {}", inbox.id);
}
