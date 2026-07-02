use integration_core::{ConnectorCapability, ConnectorDescriptor, native_connector_catalog};
use platform_core::DataClass;
use workflow_engine::{ApprovalGate, WorkflowAction, WorkflowDefinition, WorkflowTrigger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationOutcome {
    IncidentContainment,
    AccessProvisioning,
    VulnerabilityRemediation,
    ChangeApproval,
    VendorEscalation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrationProfile {
    pub connector_id: &'static str,
    pub required_capabilities: &'static [ConnectorCapability],
    pub min_data_classes: Vec<DataClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionGuardrail {
    pub name: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationTemplate {
    pub id: &'static str,
    pub display_name: &'static str,
    pub outcome: AutomationOutcome,
    pub vendor_profiles: Vec<IntegrationProfile>,
    pub production_guardrails: Vec<ProductionGuardrail>,
    pub workflow: WorkflowDefinition,
}

impl AutomationTemplate {
    pub fn is_enterprise_ready(&self) -> bool {
        self.production_guardrails
            .iter()
            .filter(|guardrail| guardrail.required)
            .count()
            >= 3
    }

    pub fn compatible_connectors(&self) -> Vec<ConnectorDescriptor> {
        let catalog = native_connector_catalog();
        catalog
            .into_iter()
            .filter(|descriptor| {
                self.vendor_profiles.iter().any(|profile| {
                    profile.connector_id == descriptor.id
                        && profile
                            .required_capabilities
                            .iter()
                            .all(|capability| descriptor.capabilities.contains(capability))
                })
            })
            .collect()
    }
}

pub fn default_automation_library() -> Vec<AutomationTemplate> {
    vec![
        AutomationTemplate {
            id: "m365-major-incident-bridge",
            display_name: "Microsoft 365 major incident bridge",
            outcome: AutomationOutcome::IncidentContainment,
            vendor_profiles: vec![IntegrationProfile {
                connector_id: "microsoft-365",
                required_capabilities: &[
                    ConnectorCapability::IdentitySync,
                    ConnectorCapability::WorkflowAutomation,
                ],
                min_data_classes: vec![DataClass::Operational],
            }],
            production_guardrails: vec![
                ProductionGuardrail {
                    name: "approval-gate",
                    required: true,
                },
                ProductionGuardrail {
                    name: "audit-log",
                    required: true,
                },
                ProductionGuardrail {
                    name: "vendor-webhook-validation",
                    required: true,
                },
            ],
            workflow: WorkflowDefinition {
                id: "wf-major-incident-bridge".to_string(),
                name: "Major incident bridge".to_string(),
                trigger: WorkflowTrigger::EventIngested,
                actions: vec![
                    WorkflowAction::CreateIncident,
                    WorkflowAction::SendNotification,
                    WorkflowAction::CreateTask,
                ],
                approvals: vec![ApprovalGate {
                    id: "approval-major-incident".to_string(),
                    approver_role: "incident-manager".to_string(),
                    approved_at: None,
                }],
            },
        },
        AutomationTemplate {
            id: "okta-access-request-fulfillment",
            display_name: "Okta access request fulfillment",
            outcome: AutomationOutcome::AccessProvisioning,
            vendor_profiles: vec![IntegrationProfile {
                connector_id: "okta",
                required_capabilities: &[
                    ConnectorCapability::IdentitySync,
                    ConnectorCapability::WorkflowAutomation,
                ],
                min_data_classes: vec![DataClass::Internal],
            }],
            production_guardrails: vec![
                ProductionGuardrail {
                    name: "manager-approval",
                    required: true,
                },
                ProductionGuardrail {
                    name: "least-privilege-check",
                    required: true,
                },
                ProductionGuardrail {
                    name: "audit-log",
                    required: true,
                },
            ],
            workflow: WorkflowDefinition {
                id: "wf-okta-access-request".to_string(),
                name: "Okta access request".to_string(),
                trigger: WorkflowTrigger::RecordCreated,
                actions: vec![
                    WorkflowAction::CreateTask,
                    WorkflowAction::CallWebhook,
                    WorkflowAction::SendNotification,
                ],
                approvals: vec![ApprovalGate {
                    id: "approval-access-request".to_string(),
                    approver_role: "manager".to_string(),
                    approved_at: None,
                }],
            },
        },
        AutomationTemplate {
            id: "critical-vuln-remediation",
            display_name: "Critical vulnerability remediation",
            outcome: AutomationOutcome::VulnerabilityRemediation,
            vendor_profiles: vec![
                IntegrationProfile {
                    connector_id: "microsoft-365",
                    required_capabilities: &[
                        ConnectorCapability::WorkflowAutomation,
                        ConnectorCapability::SecurityFindingIngestion,
                    ],
                    min_data_classes: vec![DataClass::Regulated],
                },
                IntegrationProfile {
                    connector_id: "google-workspace",
                    required_capabilities: &[
                        ConnectorCapability::WorkflowAutomation,
                        ConnectorCapability::AssetSync,
                    ],
                    min_data_classes: vec![DataClass::Operational],
                },
            ],
            production_guardrails: vec![
                ProductionGuardrail {
                    name: "two-person-approval",
                    required: true,
                },
                ProductionGuardrail {
                    name: "rollback-runbook",
                    required: true,
                },
                ProductionGuardrail {
                    name: "audit-log",
                    required: true,
                },
                ProductionGuardrail {
                    name: "maintenance-window-check",
                    required: true,
                },
            ],
            workflow: WorkflowDefinition {
                id: "wf-critical-vuln-remediation".to_string(),
                name: "Critical vulnerability remediation".to_string(),
                trigger: WorkflowTrigger::EventIngested,
                actions: vec![
                    WorkflowAction::CreateIncident,
                    WorkflowAction::CreateTask,
                    WorkflowAction::CallWebhook,
                    WorkflowAction::SendNotification,
                ],
                approvals: vec![ApprovalGate {
                    id: "approval-vuln-remediation".to_string(),
                    approver_role: "security-lead".to_string(),
                    approved_at: None,
                }],
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::default_automation_library;

    #[test]
    fn ships_multiple_enterprise_ready_automation_templates() {
        let library = default_automation_library();

        assert!(library.len() >= 3);
        assert!(
            library
                .iter()
                .all(|template| template.is_enterprise_ready())
        );
    }

    #[test]
    fn templates_reference_known_connector_profiles() {
        let library = default_automation_library();
        let major_incident = library
            .iter()
            .find(|template| template.id == "m365-major-incident-bridge")
            .expect("major incident template should exist");

        let compatible = major_incident.compatible_connectors();
        assert!(
            compatible
                .iter()
                .any(|connector| connector.id == "microsoft-365")
        );
    }
}
