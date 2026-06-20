use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{StepId, workflow::validate_workflow_name};

/// Replay schema version emitted by current runtime runs.
pub const CURRENT_REPLAY_SCHEMA_VERSION: u32 = 1;
/// Schema version assigned to legacy replay files without an explicit version.
pub const LEGACY_REPLAY_SCHEMA_VERSION: u32 = 0;

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
/// Non-secret runtime provenance recorded with a replay.
pub struct RuntimeMetadata {
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    /// Provider family used for the run.
    pub provider: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    /// Adapter implementation that produced the run.
    pub adapter: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    /// Adapter crate or implementation version.
    pub adapter_version: String,
    #[serde(default, deserialize_with = "deserialize_optional_non_empty_string")]
    /// Model identifier, when the adapter uses one.
    pub model: Option<String>,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    /// Non-secret adapter identity used to scope runtime cache entries.
    pub cache_identity: String,
    #[serde(default, deserialize_with = "deserialize_parameters")]
    /// Additional non-secret provider or runtime parameters.
    pub parameters: BTreeMap<String, String>,
}

impl RuntimeMetadata {
    /// Creates runtime metadata from required non-secret provenance fields.
    pub fn new(
        provider: impl Into<String>,
        adapter: impl Into<String>,
        adapter_version: impl Into<String>,
        cache_identity: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            adapter: adapter.into(),
            adapter_version: adapter_version.into(),
            model: None,
            cache_identity: cache_identity.into(),
            parameters: BTreeMap::new(),
        }
    }

    /// Adds a model identifier.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Adds one non-secret runtime parameter.
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    fn legacy() -> Self {
        Self::new("legacy", "unknown", "unknown", "legacy")
    }
}

impl Default for RuntimeMetadata {
    fn default() -> Self {
        Self::legacy()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Deterministic replay report produced by a workflow run.
pub struct RunReport {
    #[serde(
        default = "legacy_replay_schema_version",
        deserialize_with = "deserialize_replay_schema_version"
    )]
    /// Replay schema version.
    pub schema_version: u32,
    #[serde(deserialize_with = "deserialize_workflow_name")]
    /// Workflow name associated with this run.
    pub workflow_name: String,
    #[serde(default)]
    /// Non-secret runtime provenance for this run.
    pub runtime: RuntimeMetadata,
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
    /// Runtime metadata differs between the expected replay and actual run.
    RuntimeMetadata {
        /// Runtime metadata from the expected replay.
        expected: Box<RuntimeMetadata>,
        /// Runtime metadata from the actual run.
        actual: Box<RuntimeMetadata>,
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
            | ReplayMismatch::RuntimeMetadata { .. }
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

fn legacy_replay_schema_version() -> u32 {
    LEGACY_REPLAY_SCHEMA_VERSION
}

fn deserialize_replay_schema_version<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;

    match value {
        LEGACY_REPLAY_SCHEMA_VERSION | CURRENT_REPLAY_SCHEMA_VERSION => Ok(value),
        _ => Err(de::Error::custom(format!(
            "unsupported replay schema_version `{value}`; supported versions are {LEGACY_REPLAY_SCHEMA_VERSION} and {CURRENT_REPLAY_SCHEMA_VERSION}"
        ))),
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

fn deserialize_non_empty_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    reject_blank_string(&value).map_err(de::Error::custom)?;
    Ok(value)
}

fn deserialize_optional_non_empty_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;

    if let Some(value) = value.as_deref() {
        reject_blank_string(value).map_err(de::Error::custom)?;
    }

    Ok(value)
}

fn deserialize_parameters<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let parameters = BTreeMap::<String, String>::deserialize(deserializer)?;

    for (key, value) in &parameters {
        reject_blank_parameter("runtime parameter key", key).map_err(de::Error::custom)?;
        reject_blank_parameter("runtime parameter value", value).map_err(de::Error::custom)?;
    }

    Ok(parameters)
}

fn reject_blank_string(value: &str) -> std::result::Result<(), String> {
    if value.trim().is_empty() {
        Err("runtime metadata fields must not be blank".to_owned())
    } else {
        Ok(())
    }
}

fn reject_blank_parameter(name: &str, value: &str) -> std::result::Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{name} must not be blank"))
    } else {
        Ok(())
    }
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
    use crate::{CURRENT_REPLAY_SCHEMA_VERSION, LEGACY_REPLAY_SCHEMA_VERSION, RunReport};

    fn valid_step_json() -> &'static str {
        r#"{
            "step_id": "classify",
            "input_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "output_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "output": "done"
        }"#
    }

    #[test]
    fn run_report_deserialization_accepts_legacy_unversioned_replays() {
        let report = serde_json::from_str::<RunReport>(&format!(
            r#"{{
                "workflow_name": "demo",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{}]
            }}"#,
            valid_step_json()
        ))
        .unwrap();

        assert_eq!(report.schema_version, LEGACY_REPLAY_SCHEMA_VERSION);
        assert_eq!(report.runtime.provider, "legacy");
    }

    #[test]
    fn run_report_deserialization_accepts_current_runtime_metadata() {
        let report = serde_json::from_str::<RunReport>(&format!(
            r#"{{
                "schema_version": {CURRENT_REPLAY_SCHEMA_VERSION},
                "workflow_name": "demo",
                "runtime": {{
                    "provider": "deterministic",
                    "adapter": "deterministic-echo",
                    "adapter_version": "0.1.0",
                    "model": "deterministic-echo",
                    "cache_identity": "vogon-adapters@0.1.0:deterministic-echo:v1",
                    "parameters": {{
                        "mode": "offline"
                    }}
                }},
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{}]
            }}"#,
            valid_step_json()
        ))
        .unwrap();

        assert_eq!(report.schema_version, CURRENT_REPLAY_SCHEMA_VERSION);
        assert_eq!(report.runtime.provider, "deterministic");
        assert_eq!(report.runtime.model.as_deref(), Some("deterministic-echo"));
        assert_eq!(
            report.runtime.parameters.get("mode").map(String::as_str),
            Some("offline")
        );
    }

    #[test]
    fn run_report_deserialization_rejects_unsupported_schema_versions() {
        let result = serde_json::from_str::<RunReport>(&format!(
            r#"{{
                "schema_version": 99,
                "workflow_name": "demo",
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{}]
            }}"#,
            valid_step_json()
        ));

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unsupported replay schema_version `99`; supported versions are 0 and 1")
        );
    }

    #[test]
    fn run_report_deserialization_rejects_blank_runtime_metadata() {
        let result = serde_json::from_str::<RunReport>(&format!(
            r#"{{
                "schema_version": {CURRENT_REPLAY_SCHEMA_VERSION},
                "workflow_name": "demo",
                "runtime": {{
                    "provider": " ",
                    "adapter": "deterministic-echo",
                    "adapter_version": "0.1.0",
                    "model": "deterministic-echo",
                    "cache_identity": "vogon-adapters@0.1.0:deterministic-echo:v1"
                }},
                "run_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "steps": [{}]
            }}"#,
            valid_step_json()
        ));

        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("runtime metadata fields must not be blank")
        );
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
