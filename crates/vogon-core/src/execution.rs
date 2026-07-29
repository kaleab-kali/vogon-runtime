use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Result, RuntimeMetadata, VogonError, stable_hash};

/// Maximum per-step output limit accepted in a workflow execution policy.
pub const MAX_STEP_OUTPUT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Provider-neutral restrictions applied before and during workflow execution.
pub struct ExecutionPolicy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Exact provider names allowed to receive workflow inputs.
    pub allowed_providers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Exact model names allowed to execute the workflow.
    pub allowed_models: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Maximum raw UTF-8 output bytes accepted from any step.
    pub max_step_output_bytes: Option<usize>,
}

impl ExecutionPolicy {
    /// Validates configured allowlists and output bounds.
    pub fn validate(&self) -> Result<()> {
        if self.allowed_providers.is_empty()
            && self.allowed_models.is_empty()
            && self.max_step_output_bytes.is_none()
        {
            return Err(VogonError::EmptyExecutionPolicy);
        }
        validate_values("allowed_providers", &self.allowed_providers)?;
        validate_values("allowed_models", &self.allowed_models)?;
        if let Some(limit) = self.max_step_output_bytes {
            if limit == 0 || limit > MAX_STEP_OUTPUT_BYTES {
                return Err(VogonError::InvalidStepOutputLimit {
                    value: limit,
                    maximum: MAX_STEP_OUTPUT_BYTES,
                });
            }
        }
        Ok(())
    }

    /// Enforces provider and model allowlists before adapter execution.
    pub fn validate_runtime(&self, runtime: &RuntimeMetadata) -> Result<()> {
        if !self.allowed_providers.is_empty()
            && !self
                .allowed_providers
                .iter()
                .any(|provider| provider == &runtime.provider)
        {
            return Err(VogonError::ProviderNotAllowed(runtime.provider.clone()));
        }
        if !self.allowed_models.is_empty()
            && !runtime
                .model
                .as_ref()
                .is_some_and(|model| self.allowed_models.iter().any(|allowed| allowed == model))
        {
            return Err(VogonError::ModelNotAllowed(
                runtime
                    .model
                    .clone()
                    .unwrap_or_else(|| "<unspecified>".to_owned()),
            ));
        }
        Ok(())
    }

    /// Returns a stable identity for the policy's effective restrictions.
    pub fn policy_hash(&self) -> String {
        let mut material = String::new();
        let mut providers = self.allowed_providers.iter().collect::<Vec<_>>();
        providers.sort();
        for provider in providers {
            material.push_str(&length_prefixed(provider));
        }
        material.push('|');
        let mut models = self.allowed_models.iter().collect::<Vec<_>>();
        models.sort();
        for model in models {
            material.push_str(&length_prefixed(model));
        }
        material.push('|');
        if let Some(limit) = self.max_step_output_bytes {
            material.push_str(&limit.to_string());
        }
        stable_hash(material)
    }
}

fn length_prefixed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn validate_values(field: &str, values: &[String]) -> Result<()> {
    let mut seen = HashSet::new();
    for value in values {
        if value.is_empty() || value != value.trim() {
            return Err(VogonError::InvalidExecutionPolicyValue {
                field: field.to_owned(),
                value: value.clone(),
            });
        }
        if !seen.insert(value) {
            return Err(VogonError::DuplicateExecutionPolicyValue {
                field: field.to_owned(),
                value: value.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ExecutionPolicy, MAX_STEP_OUTPUT_BYTES};
    use crate::{RuntimeMetadata, VogonError};

    fn policy() -> ExecutionPolicy {
        ExecutionPolicy {
            allowed_providers: vec!["nvidia".to_owned()],
            allowed_models: vec!["meta/model".to_owned()],
            max_step_output_bytes: Some(4096),
        }
    }

    #[test]
    fn validates_runtime_provider_and_model() {
        let runtime =
            RuntimeMetadata::new("nvidia", "adapter", "1", "cache").with_model("meta/model");
        assert!(policy().validate_runtime(&runtime).is_ok());

        let wrong_provider =
            RuntimeMetadata::new("openrouter", "adapter", "1", "cache").with_model("meta/model");
        assert_eq!(
            policy().validate_runtime(&wrong_provider).unwrap_err(),
            VogonError::ProviderNotAllowed("openrouter".to_owned())
        );

        let wrong_model =
            RuntimeMetadata::new("nvidia", "adapter", "1", "cache").with_model("other/model");
        assert_eq!(
            policy().validate_runtime(&wrong_model).unwrap_err(),
            VogonError::ModelNotAllowed("other/model".to_owned())
        );

        let missing_model = RuntimeMetadata::new("nvidia", "adapter", "1", "cache");
        assert_eq!(
            policy().validate_runtime(&missing_model).unwrap_err(),
            VogonError::ModelNotAllowed("<unspecified>".to_owned())
        );
    }

    #[test]
    fn rejects_empty_duplicate_and_oversized_policies() {
        let empty = ExecutionPolicy {
            allowed_providers: Vec::new(),
            allowed_models: Vec::new(),
            max_step_output_bytes: None,
        };
        assert_eq!(
            empty.validate().unwrap_err(),
            VogonError::EmptyExecutionPolicy
        );

        let mut duplicate = policy();
        duplicate.allowed_models.push("meta/model".to_owned());
        assert!(matches!(
            duplicate.validate(),
            Err(VogonError::DuplicateExecutionPolicyValue { .. })
        ));

        let mut oversized = policy();
        oversized.max_step_output_bytes = Some(MAX_STEP_OUTPUT_BYTES + 1);
        assert!(matches!(
            oversized.validate(),
            Err(VogonError::InvalidStepOutputLimit { .. })
        ));
    }

    #[test]
    fn policy_hash_ignores_allowlist_order() {
        let mut reordered = policy();
        reordered.allowed_providers.push("openrouter".to_owned());
        reordered.allowed_models.push("other/model".to_owned());
        let mut reversed = reordered.clone();
        reversed.allowed_providers.reverse();
        reversed.allowed_models.reverse();

        assert_eq!(reordered.policy_hash(), reversed.policy_hash());
    }
}
