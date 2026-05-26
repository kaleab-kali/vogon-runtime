use thiserror::Error;

pub type Result<T> = std::result::Result<T, VogonError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VogonError {
    #[error("workflow name cannot be empty")]
    EmptyWorkflowName,

    #[error("workflow must contain at least one step")]
    EmptyWorkflow,

    #[error("step id cannot be empty")]
    EmptyStepId,

    #[error("step id `{0}` contains unsupported characters")]
    InvalidStepId(String),

    #[error("duplicate step id `{0}`")]
    DuplicateStepId(String),

    #[error("model adapter failed: {0}")]
    Adapter(String),
}
