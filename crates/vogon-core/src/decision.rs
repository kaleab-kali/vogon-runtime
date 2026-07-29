use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{Result, StepId, VogonError, stable_hash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Machine-enforceable decision policy attached to a workflow.
pub struct DecisionPolicy {
    /// Final workflow step whose output contains the decision.
    pub step: StepId,
    /// JSON Pointer selecting the decision string from the step output.
    pub pointer: String,
    /// Exact values that allow the workflow gate to pass.
    pub allow: Vec<String>,
    /// Exact values that deny the workflow gate.
    pub deny: Vec<String>,
}

impl DecisionPolicy {
    /// Validates the decision shape and its position in the workflow.
    pub fn validate(&self, final_step: &StepId) -> Result<()> {
        if &self.step != final_step {
            return Err(VogonError::DecisionStepNotFinal {
                configured: self.step.as_str().to_owned(),
                final_step: final_step.as_str().to_owned(),
            });
        }
        validate_json_pointer(&self.pointer)?;
        validate_values("allow", &self.allow)?;
        validate_values("deny", &self.deny)?;

        let allowed = self.allow.iter().collect::<HashSet<_>>();
        if let Some(value) = self.deny.iter().find(|value| allowed.contains(value)) {
            return Err(VogonError::OverlappingDecisionValue((*value).clone()));
        }

        Ok(())
    }

    /// Evaluates one strict JSON output against this policy.
    pub fn evaluate(&self, output: &str) -> Result<DecisionResult> {
        let document = serde_json::from_str::<serde_json::Value>(output).map_err(|error| {
            VogonError::InvalidDecisionJson {
                step_id: self.step.as_str().to_owned(),
                message: error.to_string(),
            }
        })?;
        if !document.is_object() {
            return Err(VogonError::DecisionJsonMustBeObject(
                self.step.as_str().to_owned(),
            ));
        }
        let selected =
            document
                .pointer(&self.pointer)
                .ok_or_else(|| VogonError::MissingDecisionField {
                    step_id: self.step.as_str().to_owned(),
                    pointer: self.pointer.clone(),
                })?;
        let value = selected
            .as_str()
            .ok_or_else(|| VogonError::DecisionFieldNotString {
                step_id: self.step.as_str().to_owned(),
                pointer: self.pointer.clone(),
            })?;
        let outcome = if self.allow.iter().any(|allowed| allowed == value) {
            DecisionOutcome::Allow
        } else if self.deny.iter().any(|denied| denied == value) {
            DecisionOutcome::Deny
        } else {
            return Err(VogonError::UnknownDecisionValue {
                step_id: self.step.as_str().to_owned(),
                value: value.to_owned(),
            });
        };

        Ok(DecisionResult {
            step_id: self.step.clone(),
            pointer: self.pointer.clone(),
            policy_hash: self.policy_hash(),
            value: value.to_owned(),
            outcome,
        })
    }

    /// Returns a stable identity for this exact decision policy.
    pub fn policy_hash(&self) -> String {
        let mut material = length_prefixed(self.step.as_str());
        material.push_str(&length_prefixed(&self.pointer));
        let mut allow = self.allow.iter().collect::<Vec<_>>();
        allow.sort();
        for value in allow {
            material.push_str(&length_prefixed(value));
        }
        material.push('|');
        let mut deny = self.deny.iter().collect::<Vec<_>>();
        deny.sort();
        for value in deny {
            material.push_str(&length_prefixed(value));
        }
        stable_hash(material)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Result category produced by a workflow decision policy.
pub enum DecisionOutcome {
    /// The selected decision value allows the gate to pass.
    Allow,
    /// The selected decision value denies the gate.
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// Evaluated decision evidence recorded in a replay.
pub struct DecisionResult {
    /// Step that supplied the decision document.
    pub step_id: StepId,
    /// JSON Pointer used to select the decision value.
    pub pointer: String,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    /// Stable hash binding the result to the workflow decision policy.
    pub policy_hash: String,
    /// Exact selected decision value.
    pub value: String,
    /// Whether the selected value allows or denies the gate.
    pub outcome: DecisionOutcome,
}

impl<'de> Deserialize<'de> for DecisionResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct DecisionResultFields {
            step_id: StepId,
            pointer: String,
            #[serde(deserialize_with = "deserialize_sha256_hex")]
            policy_hash: String,
            value: String,
            outcome: DecisionOutcome,
        }

        let fields = DecisionResultFields::deserialize(deserializer)?;
        validate_json_pointer(&fields.pointer).map_err(de::Error::custom)?;
        if fields.value.is_empty() || fields.value != fields.value.trim() {
            return Err(de::Error::custom(
                "decision value must be non-empty and have no surrounding whitespace",
            ));
        }

        Ok(Self {
            step_id: fields.step_id,
            pointer: fields.pointer,
            policy_hash: fields.policy_hash,
            value: fields.value,
            outcome: fields.outcome,
        })
    }
}

fn length_prefixed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn validate_json_pointer(pointer: &str) -> Result<()> {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return Err(VogonError::InvalidDecisionPointer(pointer.to_owned()));
    }

    for token in pointer.split('/').skip(1) {
        let bytes = token.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'~' {
                if bytes
                    .get(index + 1)
                    .is_none_or(|next| !matches!(next, b'0' | b'1'))
                {
                    return Err(VogonError::InvalidDecisionPointer(pointer.to_owned()));
                }
                index += 2;
            } else {
                index += 1;
            }
        }
    }
    Ok(())
}

fn validate_values(label: &str, values: &[String]) -> Result<()> {
    if values.is_empty() {
        return Err(VogonError::EmptyDecisionValues(label.to_owned()));
    }

    let mut seen = HashSet::new();
    for value in values {
        if value.is_empty() || value != value.trim() {
            return Err(VogonError::InvalidDecisionValue(value.clone()));
        }
        if !seen.insert(value) {
            return Err(VogonError::DuplicateDecisionValue(value.clone()));
        }
    }
    Ok(())
}

fn deserialize_sha256_hex<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(de::Error::custom(
            "hash must be 64 lowercase hexadecimal characters",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{DecisionOutcome, DecisionPolicy};
    use crate::{StepId, VogonError};

    fn policy() -> DecisionPolicy {
        DecisionPolicy {
            step: StepId::new("release_decision").unwrap(),
            pointer: "/decision".to_owned(),
            allow: vec!["GO".to_owned()],
            deny: vec!["NO_GO".to_owned()],
        }
    }

    #[test]
    fn evaluates_allowed_and_denied_json_decisions() {
        let policy = policy();

        let allowed = policy
            .evaluate(r#"{"decision":"GO","reasons":[]}"#)
            .unwrap();
        let denied = policy.evaluate(r#"{"decision":"NO_GO"}"#).unwrap();

        assert_eq!(allowed.outcome, DecisionOutcome::Allow);
        assert_eq!(allowed.value, "GO");
        assert_eq!(denied.outcome, DecisionOutcome::Deny);
        assert_eq!(allowed.policy_hash, policy.policy_hash());
    }

    #[test]
    fn rejects_markdown_wrapped_or_unknown_decisions() {
        let policy = policy();

        assert!(matches!(
            policy.evaluate("```json\n{\"decision\":\"GO\"}\n```"),
            Err(VogonError::InvalidDecisionJson { .. })
        ));
        assert_eq!(
            policy.evaluate(r#"{"decision":"MAYBE"}"#).unwrap_err(),
            VogonError::UnknownDecisionValue {
                step_id: "release_decision".to_owned(),
                value: "MAYBE".to_owned(),
            }
        );
    }

    #[test]
    fn validates_pointer_values_and_final_step() {
        let final_step = StepId::new("release_decision").unwrap();
        assert!(policy().validate(&final_step).is_ok());

        let mut invalid = policy();
        invalid.pointer = "decision".to_owned();
        assert_eq!(
            invalid.validate(&final_step).unwrap_err(),
            VogonError::InvalidDecisionPointer("decision".to_owned())
        );

        let mut overlapping = policy();
        overlapping.deny.push("GO".to_owned());
        assert_eq!(
            overlapping.validate(&final_step).unwrap_err(),
            VogonError::OverlappingDecisionValue("GO".to_owned())
        );

        assert!(matches!(
            policy().validate(&StepId::new("later").unwrap()),
            Err(VogonError::DecisionStepNotFinal { .. })
        ));
    }

    #[test]
    fn policy_hash_uses_value_set_semantics() {
        let mut first = policy();
        first.allow.push("APPROVED".to_owned());
        first.deny.push("BLOCK".to_owned());
        let mut reordered = first.clone();
        reordered.allow.reverse();
        reordered.deny.reverse();

        assert_eq!(first.policy_hash(), reordered.policy_hash());
    }

    #[test]
    fn decision_result_deserialization_rejects_invalid_evidence() {
        let invalid = format!(
            r#"{{
                "step_id": "decide",
                "pointer": "decision",
                "policy_hash": "{}",
                "value": "GO",
                "outcome": "allow"
            }}"#,
            "0".repeat(64)
        );

        assert!(
            serde_json::from_str::<super::DecisionResult>(&invalid)
                .unwrap_err()
                .to_string()
                .contains("valid JSON Pointer")
        );
    }
}
