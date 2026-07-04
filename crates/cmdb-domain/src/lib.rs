use std::collections::{HashSet, VecDeque};

use platform_core::{DataClass, ExternalRef, TeamId, TenantId};
use platform_domain::Criticality;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigurationItemId(pub String);

impl ConfigurationItemId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelationshipId(pub String);

impl RelationshipId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationItemClass {
    Server,
    Database,
    Application,
    Endpoint,
    NetworkDevice,
    CloudResource,
    IdentityProvider,
    BusinessService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationItemStatus {
    Planned,
    Live,
    Degraded,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    DependsOn,
    HostedOn,
    ConnectedTo,
    OwnedBy,
    Supports,
    ImpactedBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationItem {
    pub id: ConfigurationItemId,
    pub tenant_id: TenantId,
    pub name: String,
    pub class: ConfigurationItemClass,
    pub status: ConfigurationItemStatus,
    pub criticality: Criticality,
    pub owner_team_id: Option<TeamId>,
    pub environment: String,
    pub data_classes: Vec<DataClass>,
    pub external_refs: Vec<ExternalRef>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub id: RelationshipId,
    pub tenant_id: TenantId,
    pub from: ConfigurationItemId,
    pub to: ConfigurationItemId,
    pub relationship_type: RelationshipType,
    pub discovered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceGraph {
    pub items: Vec<ConfigurationItem>,
    pub relationships: Vec<Relationship>,
}

impl ServiceGraph {
    pub fn add_item(&mut self, item: ConfigurationItem) {
        if !self.items.iter().any(|current| current.id == item.id) {
            self.items.push(item);
        }
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        if !self
            .relationships
            .iter()
            .any(|current| current.id == relationship.id)
        {
            self.relationships.push(relationship);
        }
    }

    pub fn downstream_of(&self, start: &ConfigurationItemId) -> Vec<ConfigurationItemId> {
        self.walk(start, true)
    }

    pub fn upstream_of(&self, start: &ConfigurationItemId) -> Vec<ConfigurationItemId> {
        self.walk(start, false)
    }

    fn walk(&self, start: &ConfigurationItemId, forward: bool) -> Vec<ConfigurationItemId> {
        let mut queue = VecDeque::from([start.clone()]);
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();

        while let Some(current) = queue.pop_front() {
            for relationship in &self.relationships {
                let next = if forward && relationship.from == current {
                    Some(relationship.to.clone())
                } else if !forward && relationship.to == current {
                    Some(relationship.from.clone())
                } else {
                    None
                };

                if let Some(next) = next
                    && seen.insert(next.clone())
                {
                    queue.push_back(next.clone());
                    ordered.push(next);
                }
            }
        }

        ordered
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ConfigurationItem, ConfigurationItemClass, ConfigurationItemId, ConfigurationItemStatus,
        Relationship, RelationshipId, RelationshipType, ServiceGraph,
    };
    use platform_core::{DataClass, ExternalRef, TeamId, TenantId};
    use platform_domain::Criticality;

    fn item(id: &str, class: ConfigurationItemClass) -> ConfigurationItem {
        ConfigurationItem {
            id: ConfigurationItemId::new(id),
            tenant_id: TenantId::new("tenant-a"),
            name: id.to_string(),
            class,
            status: ConfigurationItemStatus::Live,
            criticality: Criticality::MissionCritical,
            owner_team_id: Some(TeamId::new("team-a")),
            environment: "prod".to_string(),
            data_classes: vec![DataClass::Operational],
            external_refs: vec![ExternalRef {
                system: "discovery".to_string(),
                external_id: id.to_string(),
                url: None,
            }],
            tags: vec!["prod".to_string()],
        }
    }

    #[test]
    fn traverses_downstream_dependencies() {
        let mut graph = ServiceGraph::default();
        graph.add_item(item("svc-mail", ConfigurationItemClass::BusinessService));
        graph.add_item(item("app-mail", ConfigurationItemClass::Application));
        graph.add_item(item("db-mail", ConfigurationItemClass::Database));
        graph.add_relationship(Relationship {
            id: RelationshipId::new("rel-1"),
            tenant_id: TenantId::new("tenant-a"),
            from: ConfigurationItemId::new("svc-mail"),
            to: ConfigurationItemId::new("app-mail"),
            relationship_type: RelationshipType::DependsOn,
            discovered: true,
        });
        graph.add_relationship(Relationship {
            id: RelationshipId::new("rel-2"),
            tenant_id: TenantId::new("tenant-a"),
            from: ConfigurationItemId::new("app-mail"),
            to: ConfigurationItemId::new("db-mail"),
            relationship_type: RelationshipType::DependsOn,
            discovered: true,
        });

        let downstream = graph.downstream_of(&ConfigurationItemId::new("svc-mail"));
        assert_eq!(downstream.len(), 2);
        assert_eq!(downstream[0], ConfigurationItemId::new("app-mail"));
        assert_eq!(downstream[1], ConfigurationItemId::new("db-mail"));
    }

    #[test]
    fn traverses_upstream_dependents() {
        let mut graph = ServiceGraph::default();
        graph.add_item(item("svc-mail", ConfigurationItemClass::BusinessService));
        graph.add_item(item("app-mail", ConfigurationItemClass::Application));
        graph.add_item(item("db-mail", ConfigurationItemClass::Database));
        graph.add_relationship(Relationship {
            id: RelationshipId::new("rel-1"),
            tenant_id: TenantId::new("tenant-a"),
            from: ConfigurationItemId::new("svc-mail"),
            to: ConfigurationItemId::new("app-mail"),
            relationship_type: RelationshipType::DependsOn,
            discovered: true,
        });
        graph.add_relationship(Relationship {
            id: RelationshipId::new("rel-2"),
            tenant_id: TenantId::new("tenant-a"),
            from: ConfigurationItemId::new("app-mail"),
            to: ConfigurationItemId::new("db-mail"),
            relationship_type: RelationshipType::DependsOn,
            discovered: true,
        });

        let upstream = graph.upstream_of(&ConfigurationItemId::new("db-mail"));
        assert_eq!(upstream.len(), 2);
        assert_eq!(upstream[0], ConfigurationItemId::new("app-mail"));
        assert_eq!(upstream[1], ConfigurationItemId::new("svc-mail"));
    }
}
