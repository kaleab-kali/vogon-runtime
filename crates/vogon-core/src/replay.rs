use serde::{Deserialize, Serialize};

use crate::StepId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: StepId,
    pub input_hash: String,
    pub output_hash: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunReport {
    pub workflow_name: String,
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
