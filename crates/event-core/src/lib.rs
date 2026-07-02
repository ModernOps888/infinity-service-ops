use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMetadata {
    pub event_id: String,
    pub event_type: String,
    pub schema_version: u16,
    pub occurred_at: String,
    pub actor_id: ActorId,
    pub tenant_id: TenantId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<String>,
    pub data_class: DataClass,
}

impl EventMetadata {
    pub fn new(
        event_type: impl Into<String>,
        actor_id: ActorId,
        tenant_id: TenantId,
        correlation_id: CorrelationId,
        data_class: DataClass,
    ) -> Self {
        let event_type = event_type.into();
        let event_id = format!("{}:{}", tenant_id.0, event_type);

        Self {
            event_id,
            event_type,
            schema_version: 1,
            occurred_at: "1970-01-01T00:00:00Z".to_string(),
            actor_id,
            tenant_id,
            correlation_id,
            causation_id: None,
            data_class,
        }
    }

    pub fn integrity_subject(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.tenant_id.0, self.event_type, self.schema_version, self.correlation_id.0
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope<T> {
    pub metadata: EventMetadata,
    pub payload: T,
    pub payload_fingerprint: String,
}

impl<T> EventEnvelope<T> {
    pub fn new(
        metadata: EventMetadata,
        payload: T,
        payload_fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            metadata,
            payload,
            payload_fingerprint: payload_fingerprint.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EventEnvelope, EventMetadata};
    use platform_core::{ActorId, CorrelationId, DataClass, TenantId};

    #[test]
    fn event_metadata_carries_tenant_and_correlation() {
        let metadata = EventMetadata::new(
            "incident.created",
            ActorId::new("api"),
            TenantId::new("tenant-a"),
            CorrelationId::new("corr-1"),
            DataClass::Operational,
        );

        assert_eq!(metadata.event_type, "incident.created");
        assert_eq!(metadata.tenant_id.0, "tenant-a");
        assert_eq!(metadata.correlation_id.0, "corr-1");
        assert_eq!(
            metadata.integrity_subject(),
            "tenant-a:incident.created:1:corr-1"
        );
    }

    #[test]
    fn envelope_keeps_payload_fingerprint() {
        let metadata = EventMetadata::new(
            "ticket.updated",
            ActorId::new("worker"),
            TenantId::new("tenant-a"),
            CorrelationId::new("corr-2"),
            DataClass::Internal,
        );

        let envelope = EventEnvelope::new(metadata, "payload", "hash-001");
        assert_eq!(envelope.payload_fingerprint, "hash-001");
    }
}
