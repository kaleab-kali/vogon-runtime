use crate::StepId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuntimeEvent {
    StepStarted { step_id: StepId },
    StepFinished { step_id: StepId },
    ReplayMismatch { step_id: Option<StepId> },
}
