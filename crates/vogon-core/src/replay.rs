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
