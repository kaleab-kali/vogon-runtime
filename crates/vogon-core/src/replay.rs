use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{StepId, workflow::validate_workflow_name};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Recorded output and hashes for one workflow step.
pub struct StepResult {
    /// Identifier of the workflow step that produced this result.
    pub step_id: StepId,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    /// Stable hash of the prompt input sent to the adapter.
    pub input_hash: String,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    /// Stable hash of the recorded output after redaction.
    pub output_hash: String,
    /// Recorded step output, redacted when redactions were configured.
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Deterministic replay report produced by a workflow run.
pub struct RunReport {
    #[serde(deserialize_with = "deserialize_workflow_name")]
    /// Workflow name associated with this run.
    pub workflow_name: String,
    #[serde(deserialize_with = "deserialize_sha256_hex")]
    /// Stable hash of the ordered step identifiers and step hashes.
    pub run_hash: String,
    #[serde(deserialize_with = "deserialize_non_empty_steps")]
    /// Ordered step results recorded during the run.
    pub steps: Vec<StepResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
/// Difference between an expected replay and an actual workflow run.
pub enum ReplayMismatch {
    /// The expected and actual workflow names differ.
    WorkflowName {
        /// Workflow name from the expected replay.
        expected: String,
        /// Workflow name from the actual run.
        actual: String,
    },
    /// The expected and actual run hashes differ.
    RunHash {
        /// Run hash from the expected replay.
        expected: String,
        /// Run hash from the actual run.
        actual: String,
    },
    /// The expected and actual step counts differ.
    StepCount {
        /// Number of steps in the expected replay.
        expected: usize,
        /// Number of steps in the actual run.
        actual: usize,
    },
    /// Step identifiers differ at a shared index.
    StepId {
        /// Zero-based step index where the mismatch was found.
        index: usize,
        /// Step identifier from the expected replay.
        expected: StepId,
        /// Step identifier from the actual run.
        actual: StepId,
    },
    /// Input hashes differ for a step.
    StepInputHash {
        /// Step identifier from the actual run.
        step_id: StepId,
        /// Input hash from the expected replay.
        expected: String,
        /// Input hash from the actual run.
        actual: String,
    },
    /// Output hashes differ for a step.
    StepOutputHash {
        /// Step identifier from the actual run.
        step_id: StepId,
        /// Output hash from the expected replay.
        expected: String,
        /// Output hash from the actual run.
        actual: String,
    },
    /// Output text differs for a step.
    StepOutput {
        /// Step identifier from the actual run.
        step_id: StepId,
        /// Output text from the expected replay.
        expected: String,
        /// Output text from the actual run.
        actual: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Verification result for comparing a workflow run to an expected replay.
pub struct VerificationReport {
    /// Workflow name from the actual run.
    pub workflow_name: String,
    /// Mismatches found while comparing the expected and actual reports.
    pub mismatches: Vec<ReplayMismatch>,
}

impl VerificationReport {
    /// Returns true when no replay mismatches were found.
    pub fn is_match(&self) -> bool {
        self.mismatches.is_empty()
    }
}

impl ReplayMismatch {
    /// Returns the step id associated with a mismatch, when one exists.
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

fn deserialize_non_empty_steps<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<StepResult>, D::Error>
where
    D: Deserializer<'de>,
{
    let steps = Vec::<StepResult>::deserialize(deserializer)?;

    if steps.is_empty() {
        return Err(de::Error::custom("replay must contain at least one step"));
    }

    let mut step_ids = HashSet::new();
    for step in &steps {
        let step_id = step.step_id.as_str();
        if !step_ids.insert(step_id) {
            return Err(de::Error::custom(format!(
                "replay contains duplicate step id `{step_id}`"
            )));
        }
    }

    Ok(steps)
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

    fn valid_step_json() -> &'static str {
        r#"{
            "step_id": "classify",
            "input_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "output_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "output": "done"
        }"#
    }

    #[test]
    fn run_report_deserialization_rejects_malformed_workflow_names() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "support triage",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{
                    "step_id": "classify",
                    "input_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "output_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "output": "done"
                }]
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
        let result = serde_json::from_str::<RunReport>(&format!(
            r#"{{
                "workflow_name": "demo",
                "run_hash": "not-a-sha256-hash",
                "steps": [{}]
            }}"#,
            valid_step_json()
        ));

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

    #[test]
    fn run_report_deserialization_rejects_empty_steps() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "demo",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": []
            }"#,
        );

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("replay must contain at least one step")
        );
    }

    #[test]
    fn run_report_deserialization_rejects_duplicate_step_ids() {
        let result = serde_json::from_str::<RunReport>(
            r#"{
                "workflow_name": "demo",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [
                    {
                        "step_id": "draft",
                        "input_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                        "output_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                        "output": "first"
                    },
                    {
                        "step_id": "draft",
                        "input_hash": "1111111111111111111111111111111111111111111111111111111111111111",
                        "output_hash": "1111111111111111111111111111111111111111111111111111111111111111",
                        "output": "second"
                    }
                ]
            }"#,
        );

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("replay contains duplicate step id `draft`")
        );
    }
}
