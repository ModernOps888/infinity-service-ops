use platform_domain::SovereigntyPolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataClass {
    Metadata,
    Operational,
    Regulated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiCapability {
    pub name: &'static str,
    pub requires_human_approval: bool,
    pub data_classes: Vec<DataClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiProviderPolicy {
    pub allow_external_inference: bool,
    pub redact_secrets: bool,
    pub retain_prompts: bool,
}

impl AiProviderPolicy {
    pub fn internal_only() -> Self {
        Self {
            allow_external_inference: false,
            redact_secrets: true,
            retain_prompts: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiGuardrailViolation {
    ExternalInferenceBlocked,
    SecretRedactionRequired,
    PromptRetentionNotAllowed,
    ProviderNotAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiProviderKind {
    LocalModel,
    PrivateEndpoint,
    ExternalApi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiExecutionPlan {
    pub provider: AiProviderKind,
    pub approval_required: bool,
    pub prompt_redaction_required: bool,
}

pub fn can_run_capability(
    _policy: &SovereigntyPolicy,
    provider_policy: &AiProviderPolicy,
    capability: &AiCapability,
) -> Result<(), AiGuardrailViolation> {
    let regulated = capability.data_classes.contains(&DataClass::Regulated);

    // Regulated data must never leave the sovereign boundary via external inference,
    // regardless of any external-training allowance.
    if regulated && provider_policy.allow_external_inference {
        return Err(AiGuardrailViolation::ExternalInferenceBlocked);
    }

    // Prompt retention creates a durable copy of regulated data outside the workflow;
    // it is not permitted for regulated capabilities.
    if regulated && provider_policy.retain_prompts {
        return Err(AiGuardrailViolation::PromptRetentionNotAllowed);
    }

    if !provider_policy.redact_secrets {
        return Err(AiGuardrailViolation::SecretRedactionRequired);
    }

    Ok(())
}

pub fn plan_execution(
    policy: &SovereigntyPolicy,
    provider_policy: &AiProviderPolicy,
    capability: &AiCapability,
    provider: AiProviderKind,
) -> Result<AiExecutionPlan, AiGuardrailViolation> {
    can_run_capability(policy, provider_policy, capability)?;

    let regulated = capability.data_classes.contains(&DataClass::Regulated);

    // Regulated capabilities can never target an external API provider, and any external
    // provider requires an explicit external-inference allowance.
    if provider == AiProviderKind::ExternalApi
        && (regulated || !provider_policy.allow_external_inference)
    {
        return Err(AiGuardrailViolation::ProviderNotAllowed);
    }

    let approval_required = capability.requires_human_approval
        || capability.data_classes.contains(&DataClass::Regulated);

    Ok(AiExecutionPlan {
        provider,
        approval_required,
        prompt_redaction_required: provider_policy.redact_secrets,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AiCapability, AiExecutionPlan, AiGuardrailViolation, AiProviderKind, AiProviderPolicy,
        DataClass, can_run_capability, plan_execution,
    };
    use platform_domain::SovereigntyPolicy;

    #[test]
    fn blocks_external_ai_for_regulated_data_when_policy_is_strict() {
        let capability = AiCapability {
            name: "change-summarizer",
            requires_human_approval: true,
            data_classes: vec![DataClass::Regulated],
        };

        let provider = AiProviderPolicy {
            allow_external_inference: true,
            redact_secrets: true,
            retain_prompts: false,
        };

        let result = can_run_capability(
            &SovereigntyPolicy::sovereign_default(),
            &provider,
            &capability,
        );

        assert_eq!(result, Err(AiGuardrailViolation::ExternalInferenceBlocked));
    }

    #[test]
    fn plans_local_execution_with_approval_for_regulated_data() {
        let capability = AiCapability {
            name: "security-case-summarizer",
            requires_human_approval: false,
            data_classes: vec![DataClass::Regulated],
        };

        let plan = plan_execution(
            &SovereigntyPolicy::sovereign_default(),
            &AiProviderPolicy::internal_only(),
            &capability,
            AiProviderKind::LocalModel,
        )
        .expect("local execution should be allowed");

        assert_eq!(
            plan,
            AiExecutionPlan {
                provider: AiProviderKind::LocalModel,
                approval_required: true,
                prompt_redaction_required: true,
            }
        );
    }

    #[test]
    fn blocks_regulated_external_api_even_when_external_training_is_allowed() {
        let capability = AiCapability {
            name: "regulated-summarizer",
            requires_human_approval: true,
            data_classes: vec![DataClass::Regulated],
        };

        let permissive_policy = SovereigntyPolicy {
            allow_external_model_training: true,
            ..SovereigntyPolicy::sovereign_default()
        };

        // Even with external inference switched on, regulated data must not reach an external API.
        let provider = AiProviderPolicy {
            allow_external_inference: true,
            redact_secrets: true,
            retain_prompts: false,
        };

        let result = plan_execution(
            &permissive_policy,
            &provider,
            &capability,
            AiProviderKind::ExternalApi,
        );

        assert_eq!(result, Err(AiGuardrailViolation::ExternalInferenceBlocked));
    }

    #[test]
    fn blocks_prompt_retention_for_regulated_data() {
        let capability = AiCapability {
            name: "regulated-triage",
            requires_human_approval: false,
            data_classes: vec![DataClass::Regulated],
        };

        let provider = AiProviderPolicy {
            allow_external_inference: false,
            redact_secrets: true,
            retain_prompts: true,
        };

        let result = can_run_capability(
            &SovereigntyPolicy::sovereign_default(),
            &provider,
            &capability,
        );

        assert_eq!(result, Err(AiGuardrailViolation::PromptRetentionNotAllowed));
    }
}
