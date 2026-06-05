use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{StepId, workflow::validate_workflow_name};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepResult {
    pub step_id: StepId,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    pub input_hash: String,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    pub output_hash: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    #[serde(deserialize_with = "deserialize_workflow_name")]
    pub workflow_name: String,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    pub run_hash: String,
    pub steps: Vec<StepResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReplayMismatch {
    WorkflowName {
        expected: String,
        actual: String,
    },
    RunHash {
        expected: String,
        actual: String,
    },
    StepCount {
        expected: usize,
        actual: usize,
    },
    StepId {
        index: usize,
        expected: StepId,
        actual: StepId,
    },
    StepInputHash {
        step_id: StepId,
        expected: String,
        actual: String,
    },
    StepOutputHash {
        step_id: StepId,
        expected: String,
        actual: String,
    },
    StepOutput {
        step_id: StepId,
        expected: String,
        actual: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationReport {
    pub workflow_name: String,
    pub mismatches: Vec<ReplayMismatch>,
}

impl VerificationReport {
    pub fn is_match(&self) -> bool {
        self.mismatches.is_empty()
    }
}

impl ReplayMismatch {
    pub fn step_id(&self) -> Option<&StepId> {
        match self {
            ReplayMismatch::WorkflowName { .. }
            | ReplayMismatch::RunHash { .. }
            | ReplayMismatch::StepCount { .. } => None,
            ReplayMismatch::StepId { actual, .. }
            | ReplayMismatch::StepInputHash {
                step_id: actual, ..
            }
            | ReplayMismatch::StepOutputHash {
                step_id: actual, ..
            }
            | ReplayMismatch::StepOutput {
                step_id: actual, ..
            } => Some(actual),
        }
    }
}

fn deserialize_workflow_name<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    validate_workflow_name(&value).map_err(de::Error::custom)?;
    Ok(value)
}

fn deserialize_sha256_hex<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    if is_sha256_hex(&value) {
        Ok(value)
    } else {
        Err(de::Error::custom(format!(
            "hash `{value}` must be 64 lowercase hexadecimal characters"
        )))
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use crate::RunReport;

    #[test]
    fn run_report_deserialization_rejects_malformed_workflow_names() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "support triage",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": []
            }"#,
        );

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("workflow name `support triage` contains unsupported characters")
        );
    }

    #[test]
    fn run_report_deserialization_rejects_malformed_run_hashes() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "demo",
                "run_hash": "not-a-sha256-hash",
                "steps": []
            }"#,
        );

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("hash `not-a-sha256-hash` must be 64 lowercase hexadecimal characters")
        );
    }

    #[test]
    fn run_report_deserialization_rejects_malformed_step_hashes() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "demo",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{
                    "step_id": "draft",
                    "input_hash": "ABC0000000000000000000000000000000000000000000000000000000000000",
                    "output_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "output": "done"
                }]
            }"#,
        );

        assert!(result.unwrap_err().to_string().contains(
            "hash `ABC0000000000000000000000000000000000000000000000000000000000000` must be 64 lowercase hexadecimal characters"
        ));
    }
}
