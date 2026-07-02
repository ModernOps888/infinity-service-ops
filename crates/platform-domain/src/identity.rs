use crate::primitives::{RecordMeta, RecordStatus, TeamId, TenantId, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStatus {
    Invited,
    Active,
    Disabled,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub external_subject: String,
    pub email: String,
    pub display_name: String,
    pub status: UserStatus,
    pub primary_team_id: Option<TeamId>,
    pub role_ids: Vec<String>,
}

impl User {
    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Active)
    }

    pub fn has_role(&self, role_id: &str) -> bool {
        self.role_ids.iter().any(|current| current == role_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub id: TeamId,
    pub tenant_id: TenantId,
    pub name: String,
    pub status: RecordStatus,
    pub manager_user_id: Option<UserId>,
    pub parent_team_id: Option<TeamId>,
    pub on_call_rotation_ref: Option<String>,
}

impl Team {
    pub fn is_active(&self) -> bool {
        self.status == RecordStatus::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ReadTicket,
    WriteTicket,
    ApproveChange,
    ManageCatalog,
    PublishKnowledge,
    ReadAudit,
    ManagePolicy,
    ManageTenant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub id: String,
    pub tenant_id: TenantId,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub meta: RecordMeta,
}

impl Role {
    pub fn grants(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTeamMembership {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub team_id: TeamId,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationContext<'a> {
    pub user: &'a User,
    pub roles: &'a [Role],
}

impl<'a> AuthorizationContext<'a> {
    pub fn new(user: &'a User, roles: &'a [Role]) -> Self {
        Self { user, roles }
    }

    pub fn allows(&self, permission: Permission) -> bool {
        self.user.is_active()
            && self
                .roles
                .iter()
                .filter(|role| self.user.has_role(&role.id))
                .any(|role| role.grants(permission))
    }
}
