use crate::primitives::{CatalogItemId, DataClass, RecordMeta, RecordStatus, TeamId, TenantId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    Planned,
    Live,
    Degraded,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceOffering {
    pub id: crate::primitives::ServiceId,
    pub tenant_id: TenantId,
    pub name: String,
    pub owner_team_id: TeamId,
    pub status: ServiceStatus,
    pub criticality: crate::sovereignty::Criticality,
    pub support_tier: String,
    pub data_classes: Vec<DataClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCatalogCategory {
    pub id: String,
    pub tenant_id: TenantId,
    pub name: String,
    pub parent_id: Option<String>,
    pub status: RecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FulfillmentKind {
    ManualTask,
    Workflow,
    IntegrationConnector,
    AiAssistedDraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogVariable {
    pub key: String,
    pub label: String,
    pub required: bool,
    pub data_class: DataClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCatalogItem {
    pub id: CatalogItemId,
    pub tenant_id: TenantId,
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub status: RecordStatus,
    pub owner_team_id: TeamId,
    pub fulfillment_kind: FulfillmentKind,
    pub fulfillment_ref: Option<String>,
    pub intake_schema_ref: Option<String>,
    pub variables: Vec<CatalogVariable>,
    pub required_policy_hook_ids: Vec<String>,
    pub meta: RecordMeta,
}
