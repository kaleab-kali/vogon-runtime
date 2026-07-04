use crate::StepId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
/// Runtime events emitted while running or verifying workflows.
pub enum RuntimeEvent {
    /// A workflow step is about to be completed.
    StepStarted {
        /// Identifier of the step that is starting.
        step_id: StepId,
    },
    /// A workflow step completed successfully.
    StepFinished {
        /// Identifier of the step that finished.
        step_id: StepId,
    },
    /// A workflow step reused an output from the run cache.
    CacheHit {
        /// Identifier of the step whose output came from cache.
        step_id: StepId,
    },
    /// A workflow step did not have a reusable output in the run cache.
    CacheMiss {
        /// Identifier of the step whose output had to be computed.
        step_id: StepId,
    },
    /// Verification found a replay mismatch.
    ReplayMismatch {
        /// Step identifier associated with the mismatch, when applicable.
        step_id: Option<StepId>,
    },
}
