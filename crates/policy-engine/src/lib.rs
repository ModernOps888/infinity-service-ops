use platform_domain::SovereigntyPolicy;
use platform_domain::SystemRecord;
use security_foundation::{SecurityBaseline, SecurityViolation, validate_system_residency};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentPlan {
    pub target_region: String,
    pub security_baseline: SecurityBaseline,
    pub systems: Vec<SystemRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyReport {
    pub violations: Vec<SecurityViolation>,
}

impl PolicyReport {
    pub fn is_approved(&self) -> bool {
        self.violations.is_empty()
    }
}

pub fn evaluate_deployment(policy: &SovereigntyPolicy, plan: &DeploymentPlan) -> PolicyReport {
    let mut violations = Vec::new();

    if !policy.allows_region(&plan.target_region) {
        violations.push(SecurityViolation::RegionNotAllowed {
            region: plan.target_region.clone(),
        });
    }

    for system in &plan.systems {
        if let Err(violation) = validate_system_residency(system, policy, &plan.security_baseline) {
            violations.push(violation);
        }
    }

    PolicyReport { violations }
}

#[cfg(test)]
mod tests {
    use super::{DeploymentPlan, evaluate_deployment};
    use platform_domain::{
        Criticality, SovereigntyMode, SovereigntyPolicy, SystemRecord, SystemSource,
    };
    use security_foundation::SecurityBaseline;

    #[test]
    fn rejects_vendor_managed_posture_for_critical_systems() {
        let policy = SovereigntyPolicy {
            mode: SovereigntyMode::VendorManaged,
            allowed_regions: vec!["eu-west".to_string()],
            requires_customer_managed_keys: false,
            allow_external_model_training: false,
        };

        let plan = DeploymentPlan {
            target_region: "eu-west".to_string(),
            security_baseline: SecurityBaseline::zero_trust(),
            systems: vec![SystemRecord {
                name: "cmdb".to_string(),
                owner_team: "operations".to_string(),
                source: SystemSource::ServiceNow,
                criticality: Criticality::MissionCritical,
                contains_regulated_data: true,
            }],
        };

        let report = evaluate_deployment(&policy, &plan);
        assert!(!report.is_approved());
    }
}
