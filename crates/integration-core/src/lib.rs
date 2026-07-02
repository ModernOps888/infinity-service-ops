use platform_domain::IntegrationTarget;
use security_foundation::{AuthMethod, AuthenticationProfile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectorCapability {
    AssetSync,
    IdentitySync,
    WorkflowAutomation,
    TicketIngestion,
    EventIngestion,
    WebhookDispatch,
    SecurityFindingIngestion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub target: IntegrationTarget,
    pub capabilities: &'static [ConnectorCapability],
    pub authentication: AuthenticationProfile,
    pub signed_manifest_required: bool,
    pub minimum_scopes: &'static [&'static str],
    pub egress_allowlist: &'static [&'static str],
}

impl ConnectorDescriptor {
    pub fn is_production_ready(&self) -> bool {
        self.signed_manifest_required
            && !self.minimum_scopes.is_empty()
            && !self.egress_allowlist.is_empty()
    }
}

pub fn native_connector_catalog() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            id: "microsoft-365",
            display_name: "Microsoft 365",
            target: IntegrationTarget::Microsoft365,
            capabilities: &[
                ConnectorCapability::AssetSync,
                ConnectorCapability::IdentitySync,
                ConnectorCapability::WorkflowAutomation,
            ],
            authentication: AuthenticationProfile {
                methods: vec![AuthMethod::HardwareBackedOidc],
                mfa_required: true,
            },
            signed_manifest_required: true,
            minimum_scopes: &["Directory.Read.All", "ServiceHealth.Read.All"],
            egress_allowlist: &["graph.microsoft.com", "manage.office.com"],
        },
        ConnectorDescriptor {
            id: "google-workspace",
            display_name: "Google Workspace",
            target: IntegrationTarget::GoogleWorkspace,
            capabilities: &[
                ConnectorCapability::AssetSync,
                ConnectorCapability::IdentitySync,
            ],
            authentication: AuthenticationProfile {
                methods: vec![AuthMethod::HardwareBackedOidc],
                mfa_required: true,
            },
            signed_manifest_required: true,
            minimum_scopes: &["admin.directory.user.readonly", "cloud-platform.read-only"],
            egress_allowlist: &["admin.googleapis.com", "cloudidentity.googleapis.com"],
        },
        ConnectorDescriptor {
            id: "okta",
            display_name: "Okta",
            target: IntegrationTarget::Okta,
            capabilities: &[ConnectorCapability::IdentitySync],
            authentication: AuthenticationProfile {
                methods: vec![AuthMethod::HardwareBackedOidc, AuthMethod::MutualTls],
                mfa_required: true,
            },
            signed_manifest_required: true,
            minimum_scopes: &["okta.users.read", "okta.groups.read"],
            egress_allowlist: &["*.okta.com"],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::native_connector_catalog;

    #[test]
    fn includes_priority_native_connectors() {
        let catalog = native_connector_catalog();
        assert!(catalog.iter().any(|item| item.id == "microsoft-365"));
        assert!(catalog.iter().any(|item| item.id == "google-workspace"));
        assert!(catalog.iter().all(|item| item.is_production_ready()));
    }
}
