use platform_domain::{Criticality, SovereigntyMode, SovereigntyPolicy, SystemRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMethod {
    Passkey,
    HardwareBackedOidc,
    MutualTls,
    ApiKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationProfile {
    pub methods: Vec<AuthMethod>,
    pub mfa_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyOwnership {
    CustomerManaged,
    PlatformManaged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionPosture {
    pub at_rest: KeyOwnership,
    pub tls13_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantBoundary {
    pub isolated_compute: bool,
    pub isolated_storage: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityBaseline {
    pub authentication: AuthenticationProfile,
    pub encryption: EncryptionPosture,
    pub tenant_boundary: TenantBoundary,
}

impl SecurityBaseline {
    pub fn zero_trust() -> Self {
        Self {
            authentication: AuthenticationProfile {
                methods: vec![AuthMethod::Passkey, AuthMethod::HardwareBackedOidc],
                mfa_required: true,
            },
            encryption: EncryptionPosture {
                at_rest: KeyOwnership::CustomerManaged,
                tls13_required: true,
            },
            tenant_boundary: TenantBoundary {
                isolated_compute: true,
                isolated_storage: true,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityViolation {
    RegionNotAllowed { region: String },
    CustomerManagedKeysRequired,
    VendorManagedCriticalSystem { system: String },
}

pub fn validate_system_residency(
    system: &SystemRecord,
    policy: &SovereigntyPolicy,
    baseline: &SecurityBaseline,
) -> Result<(), SecurityViolation> {
    if policy.requires_customer_managed_keys
        && baseline.encryption.at_rest != KeyOwnership::CustomerManaged
    {
        return Err(SecurityViolation::CustomerManagedKeysRequired);
    }

    let critical = matches!(
        system.criticality,
        Criticality::MissionCritical | Criticality::SafetyCritical
    );

    if critical && policy.mode == SovereigntyMode::VendorManaged {
        return Err(SecurityViolation::VendorManagedCriticalSystem {
            system: system.name.clone(),
        });
    }

    Ok(())
}
