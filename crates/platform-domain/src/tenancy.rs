use crate::primitives::{RecordStatus, TenantId, Timestamp};
use crate::sovereignty::{SovereigntyMode, SovereigntyPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantTier {
    Community,
    Enterprise,
    Regulated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataBoundary {
    pub primary_region: String,
    pub allowed_regions: Vec<String>,
    pub isolated_storage: bool,
    pub isolated_compute: bool,
    pub customer_key_ref: Option<String>,
}

impl DataBoundary {
    pub fn allows_region(&self, region: &str) -> bool {
        self.allowed_regions.iter().any(|allowed| allowed == region)
    }

    pub fn is_isolated(&self) -> bool {
        self.isolated_storage && self.isolated_compute
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tenant {
    pub id: TenantId,
    pub slug: String,
    pub display_name: String,
    pub tier: TenantTier,
    pub status: RecordStatus,
    pub data_boundary: DataBoundary,
    pub sovereignty_policy: SovereigntyPolicy,
    pub created_at: Timestamp,
}

impl Tenant {
    pub fn is_regulated(&self) -> bool {
        self.tier == TenantTier::Regulated
    }

    pub fn is_operational(&self) -> bool {
        self.status == RecordStatus::Active
    }

    pub fn allows_region(&self, region: &str) -> bool {
        self.data_boundary.allows_region(region) && self.sovereignty_policy.allows_region(region)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantEnvironment {
    pub id: String,
    pub tenant_id: TenantId,
    pub name: String,
    pub region: String,
    pub status: RecordStatus,
}

impl Tenant {
    pub fn regulated_default(
        id: TenantId,
        slug: String,
        display_name: String,
        region: String,
    ) -> Self {
        Self {
            id,
            slug,
            display_name,
            tier: TenantTier::Regulated,
            status: RecordStatus::Active,
            data_boundary: DataBoundary {
                primary_region: region.clone(),
                allowed_regions: vec![region.clone()],
                isolated_storage: true,
                isolated_compute: true,
                customer_key_ref: None,
            },
            sovereignty_policy: SovereigntyPolicy {
                mode: SovereigntyMode::CustomerManaged,
                allowed_regions: vec![region],
                requires_customer_managed_keys: true,
                allow_external_model_training: false,
            },
            created_at: Timestamp::new("1970-01-01T00:00:00Z"),
        }
    }
}
