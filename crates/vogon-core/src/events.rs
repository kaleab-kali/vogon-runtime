use crate::StepId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvent {
    StepStarted { step_id: StepId },
    StepFinished { step_id: StepId },
    ReplayMismatch { step_id: StepId },
}
