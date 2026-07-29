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

    /// Structural verification requires prompt hashes recorded by a newer run.
    #[error("step `{0}` has no prompt hash; create a new replay before structural verification")]
    MissingStepPromptHash(String),

    /// Decision policies must select the final workflow step.
    #[error(
        "decision step `{configured}` must be the final workflow step; final step is `{final_step}`"
    )]
    DecisionStepNotFinal {
        /// Step configured by the policy.
        configured: String,
        /// Actual final workflow step.
        final_step: String,
    },

    /// Decision JSON Pointers must use RFC 6901 syntax and select a nested field.
    #[error("decision pointer `{0}` must be a valid JSON Pointer beginning with `/`")]
    InvalidDecisionPointer(String),

    /// Decision allow and deny lists must not be empty.
    #[error("decision `{0}` values must not be empty")]
    EmptyDecisionValues(String),

    /// Decision values must be non-empty and have no surrounding whitespace.
    #[error("decision value `{0}` must be non-empty and have no surrounding whitespace")]
    InvalidDecisionValue(String),

    /// Decision values must be unique within each list.
    #[error("duplicate decision value `{0}`")]
    DuplicateDecisionValue(String),

    /// A decision value cannot both allow and deny the gate.
    #[error("decision value `{0}` cannot appear in both allow and deny lists")]
    OverlappingDecisionValue(String),

    /// Decision step output must be one strict JSON document.
    #[error("decision step `{step_id}` did not return valid JSON: {message}")]
    InvalidDecisionJson {
        /// Step that returned invalid JSON.
        step_id: String,
        /// JSON parser failure.
        message: String,
    },

    /// Decision step output must use an object at the document root.
    #[error("decision step `{0}` must return a JSON object")]
    DecisionJsonMustBeObject(String),

    /// The configured decision pointer did not resolve.
    #[error("decision step `{step_id}` has no value at JSON Pointer `{pointer}`")]
    MissingDecisionField {
        /// Step containing the decision document.
        step_id: String,
        /// Configured JSON Pointer.
        pointer: String,
    },

    /// The selected decision must be a string.
    #[error("decision step `{step_id}` value at JSON Pointer `{pointer}` must be a string")]
    DecisionFieldNotString {
        /// Step containing the decision document.
        step_id: String,
        /// Configured JSON Pointer.
        pointer: String,
    },

    /// Unlisted values fail closed.
    #[error("decision step `{step_id}` returned unrecognized value `{value}`")]
    UnknownDecisionValue {
        /// Step containing the decision document.
        step_id: String,
        /// Unrecognized value.
        value: String,
    },

    /// Decision enforcement requires a workflow policy.
    #[error("decision enforcement requires a `[decision]` workflow policy")]
    DecisionPolicyRequired,

    /// A valid deny value causes an enforced run to fail.
    #[error("workflow decision denied by step `{step_id}` with value `{value}`")]
    DecisionDenied {
        /// Step that denied the workflow.
        step_id: String,
        /// Exact deny value.
        value: String,
    },

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
