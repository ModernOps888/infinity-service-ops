use cmdb_domain::ConfigurationItemId;
use platform_core::TenantId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventStatus {
    Active,
    Suppressed,
    Correlated,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSignal {
    pub source: String,
    pub resource_id: String,
    pub title: String,
    pub description: String,
    pub severity: EventSeverity,
    pub tags: Vec<String>,
    pub occurred_at: String,
}

impl RawSignal {
    pub fn dedup_key(&self) -> String {
        format!("{}:{}:{}", self.source, self.resource_id, self.title)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEvent {
    pub id: String,
    pub tenant_id: TenantId,
    pub dedup_key: String,
    pub source: String,
    pub title: String,
    pub severity: EventSeverity,
    pub status: EventStatus,
    pub correlated_ci_id: Option<ConfigurationItemId>,
    pub raw_signal_count: u32,
}

impl NormalizedEvent {
    pub fn from_signal(
        id: impl Into<String>,
        tenant_id: TenantId,
        signal: &RawSignal,
        correlated_ci_id: Option<ConfigurationItemId>,
    ) -> Self {
        Self {
            id: id.into(),
            tenant_id,
            dedup_key: signal.dedup_key(),
            source: signal.source.clone(),
            title: signal.title.clone(),
            severity: signal.severity.clone(),
            status: EventStatus::Active,
            correlated_ci_id,
            raw_signal_count: 1,
        }
    }

    pub fn absorb(&mut self) {
        self.raw_signal_count += 1;
    }

    pub fn suppress(&mut self) {
        self.status = EventStatus::Suppressed;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceWindow {
    pub id: String,
    pub tenant_id: TenantId,
    pub resource_ids: Vec<String>,
    pub starts_at: String,
    pub ends_at: String,
}

impl MaintenanceWindow {
    pub fn suppresses(&self, signal: &RawSignal) -> bool {
        self.resource_ids
            .iter()
            .any(|resource_id| resource_id == &signal.resource_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{EventSeverity, EventStatus, MaintenanceWindow, NormalizedEvent, RawSignal};
    use cmdb_domain::ConfigurationItemId;
    use platform_core::TenantId;

    fn sample_signal() -> RawSignal {
        RawSignal {
            source: "alertmanager".to_string(),
            resource_id: "svc-mail".to_string(),
            title: "Mail service down".to_string(),
            description: "Probe failed".to_string(),
            severity: EventSeverity::Critical,
            tags: vec!["prod".to_string()],
            occurred_at: "1970-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn builds_stable_dedup_key_and_absorbs_duplicates() {
        let signal = sample_signal();
        let mut event = NormalizedEvent::from_signal(
            "evt-1",
            TenantId::new("tenant-a"),
            &signal,
            Some(ConfigurationItemId::new("svc-mail")),
        );

        assert_eq!(event.dedup_key, "alertmanager:svc-mail:Mail service down");
        event.absorb();
        assert_eq!(event.raw_signal_count, 2);
        assert_eq!(event.status, EventStatus::Active);
    }

    #[test]
    fn suppresses_events_during_maintenance_window() {
        let signal = sample_signal();
        let window = MaintenanceWindow {
            id: "mw-1".to_string(),
            tenant_id: TenantId::new("tenant-a"),
            resource_ids: vec!["svc-mail".to_string()],
            starts_at: "1970-01-01T00:00:00Z".to_string(),
            ends_at: "1970-01-01T01:00:00Z".to_string(),
        };

        let mut event =
            NormalizedEvent::from_signal("evt-1", TenantId::new("tenant-a"), &signal, None);

        if window.suppresses(&signal) {
            event.suppress();
        }

        assert_eq!(event.status, EventStatus::Suppressed);
    }
}
