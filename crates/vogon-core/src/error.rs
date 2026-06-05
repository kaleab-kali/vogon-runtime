use thiserror::Error;

pub type Result<T> = std::result::Result<T, VogonError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VogonError {
    #[error("workflow name cannot be empty")]
    EmptyWorkflowName,

    #[error("workflow name `{0}` must not have leading or trailing whitespace")]
    InvalidWorkflowName(String),

    #[error("workflow name `{0}` contains unsupported characters")]
    InvalidWorkflowNameCharacters(String),

    #[error("workflow must contain at least one step")]
    EmptyWorkflow,

    #[error("step id cannot be empty")]
    EmptyStepId,

    #[error("step `{0}` prompt cannot be empty")]
    EmptyStepPrompt(String),

    #[error("step id `{0}` contains unsupported characters")]
    InvalidStepId(String),

    #[error("duplicate step id `{0}`")]
    DuplicateStepId(String),

    #[error("redaction label cannot be empty")]
    EmptyRedactionLabel,

    #[error("redaction literal cannot be empty")]
    EmptyRedactionLiteral,

    #[error("redaction label `{0}` contains unsupported characters")]
    InvalidRedactionLabel(String),

    #[error("model adapter failed: {0}")]
    Adapter(String),
}
