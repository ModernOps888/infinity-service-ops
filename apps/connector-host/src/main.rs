use event_core::EventMetadata;
use integration_core::native_connector_catalog;
use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

fn main() {
    let catalog = native_connector_catalog();
    let selected = &catalog[0];
    let event = EventMetadata::new(
        "connector.installation.requested",
        ActorId::new("connector-host"),
        TenantId::new("tenant-demo"),
        CorrelationId::new("connector-001"),
        DataClass::Operational,
    );

    println!("Connector host bootstrap");
    println!("Selected connector: {}", selected.display_name);
    println!("Capabilities: {}", selected.capabilities.len());
    println!("Event type: {}", event.event_type);
}
