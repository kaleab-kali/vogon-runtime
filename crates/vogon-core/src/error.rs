use thiserror::Error;

/// Result type used by Vogon Runtime operations.
pub type Result<T> = std::result::Result<T, VogonError>;

/// Errors returned by workflow validation, redaction validation, and adapters.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VogonError {
    /// Workflow names must contain at least one non-whitespace character.
    #[error("workflow name cannot be empty")]
    EmptyWorkflowName,

    /// Workflow names must not have leading or trailing whitespace.
    #[error("workflow name `{0}` must not have leading or trailing whitespace")]
    InvalidWorkflowName(String),

    /// Workflow names may only contain ASCII letters, digits, underscores, and hyphens.
    #[error("workflow name `{0}` contains unsupported characters")]
    InvalidWorkflowNameCharacters(String),

    /// Workflows must contain at least one step.
    #[error("workflow must contain at least one step")]
    EmptyWorkflow,

    /// Step identifiers must contain at least one non-whitespace character.
    #[error("step id cannot be empty")]
    EmptyStepId,

    /// Step prompts must contain at least one non-whitespace character.
    #[error("step `{0}` prompt cannot be empty")]
    EmptyStepPrompt(String),

    /// Step identifiers may only contain ASCII letters, digits, underscores, and hyphens.
    #[error("step id `{0}` contains unsupported characters")]
    InvalidStepId(String),

    /// Step identifiers must be unique within a workflow.
    #[error("duplicate step id `{0}`")]
    DuplicateStepId(String),

    /// Workflow input placeholders must use a valid name and closing delimiter.
    #[error("step `{0}` contains malformed workflow input placeholder `{1}`")]
    MalformedWorkflowInputPlaceholder(String, String),

    /// Workflow input names may only contain ASCII letters, digits, underscores, and hyphens.
    #[error("workflow input name `{0}` contains unsupported characters")]
    InvalidWorkflowInputName(String),

    /// Every declared workflow input must be supplied before execution.
    #[error("workflow input `{0}` is required but was not supplied")]
    MissingWorkflowInput(String),

    /// Supplied workflow inputs must be referenced by at least one step.
    #[error("workflow input `{0}` was supplied but is not used")]
    UnusedWorkflowInput(String),

    /// Redaction labels must contain at least one non-whitespace character.
    #[error("redaction label cannot be empty")]
    EmptyRedactionLabel,

    /// Redaction literals must not be empty.
    #[error("redaction literal cannot be empty")]
    EmptyRedactionLiteral,

    /// Redaction labels must be unique within a redaction set.
    #[error("duplicate redaction label `{0}`")]
    DuplicateRedactionLabel(String),

    /// Redaction labels may only contain ASCII letters, digits, underscores, and hyphens.
    #[error("redaction label `{0}` contains unsupported characters")]
    InvalidRedactionLabel(String),

    /// A model adapter failed while completing a step.
    #[error("model adapter failed: {0}")]
    Adapter(String),
}
