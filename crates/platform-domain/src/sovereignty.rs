#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemSource {
    Microsoft365,
    GoogleWorkspace,
    ServiceNow,
    Salesforce,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationTarget {
    Microsoft365,
    GoogleWorkspace,
    Slack,
    Okta,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criticality {
    BusinessSupport,
    MissionCritical,
    SafetyCritical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SovereigntyMode {
    CustomerManaged,
    Hybrid,
    VendorManaged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRecord {
    pub name: String,
    pub owner_team: String,
    pub source: SystemSource,
    pub criticality: Criticality,
    pub contains_regulated_data: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SovereigntyPolicy {
    pub mode: SovereigntyMode,
    pub allowed_regions: Vec<String>,
    pub requires_customer_managed_keys: bool,
    pub allow_external_model_training: bool,
}

impl SovereigntyPolicy {
    pub fn sovereign_default() -> Self {
        Self {
            mode: SovereigntyMode::CustomerManaged,
            allowed_regions: vec!["eu-west".to_string()],
            requires_customer_managed_keys: true,
            allow_external_model_training: false,
        }
    }

    pub fn allows_region(&self, region: &str) -> bool {
        self.allowed_regions.iter().any(|allowed| allowed == region)
    }
}
