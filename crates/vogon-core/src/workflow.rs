use std::collections::HashSet;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{Result, Step, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
/// Ordered collection of workflow steps.
pub struct Workflow {
    /// Stable workflow name.
    pub name: String,
    /// Ordered workflow steps.
    pub steps: Vec<Step>,
}

impl Workflow {
    /// Creates and validates a workflow.
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Result<Self> {
        let workflow = Self {
            name: name.into(),
            steps,
        };
        workflow.validate()?;
        Ok(workflow)
    }

    /// Returns the workflow name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the workflow steps in execution order.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Validates workflow name, step count, duplicate step ids, and prompts.
    pub fn validate(&self) -> Result<()> {
        validate_workflow_name(&self.name)?;

        if self.steps.is_empty() {
            return Err(VogonError::EmptyWorkflow);
        }

        let mut ids = HashSet::new();
        for step in &self.steps {
            let id = step.id().as_str();
            if id.trim().is_empty() {
                return Err(VogonError::EmptyStepId);
            }

            if !ids.insert(id) {
                return Err(VogonError::DuplicateStepId(id.to_owned()));
            }

            if step.prompt().trim().is_empty() {
                return Err(VogonError::EmptyStepPrompt(id.to_owned()));
            }
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Workflow {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WorkflowFields {
            name: String,
            steps: Vec<Step>,
        }

        let fields = WorkflowFields::deserialize(deserializer)?;
        let workflow = Workflow {
            name: fields.name,
            steps: fields.steps,
        };
        workflow.validate().map_err(de::Error::custom)?;

        Ok(workflow)
    }
}

pub(crate) fn validate_workflow_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(VogonError::EmptyWorkflowName);
    }

    if name != name.trim() {
        return Err(VogonError::InvalidWorkflowName(name.to_owned()));
    }

    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(VogonError::InvalidWorkflowNameCharacters(name.to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Step, StepId, VogonError, Workflow};

    #[test]
    fn workflow_rejects_empty_names() {
        let result = Workflow::new(
            " ",
            vec![Step::new(StepId::new("classify").unwrap(), "Classify")],
        );

        assert_eq!(result.unwrap_err(), VogonError::EmptyWorkflowName);
    }

    #[test]
    fn workflow_rejects_whitespace_padded_names() {
        let result = Workflow::new(
            " support ",
            vec![Step::new(StepId::new("classify").unwrap(), "Classify")],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::InvalidWorkflowName(" support ".to_owned())
        );
    }

    #[test]
    fn workflow_rejects_names_with_unsupported_characters() {
        let result = Workflow::new(
            "support triage",
            vec![Step::new(StepId::new("classify").unwrap(), "Classify")],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::InvalidWorkflowNameCharacters("support triage".to_owned())
        );
    }

    #[test]
    fn workflow_rejects_duplicate_step_ids() {
        let result = Workflow::new(
            "support",
            vec![
                Step::new(StepId::new("classify").unwrap(), "Classify"),
                Step::new(StepId::new("classify").unwrap(), "Classify again"),
            ],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::DuplicateStepId("classify".to_owned())
        );
    }

    #[test]
    fn workflow_rejects_empty_step_prompts() {
        let result = Workflow::new(
            "support",
            vec![Step::new(StepId::new("classify").unwrap(), " ")],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::EmptyStepPrompt("classify".to_owned())
        );
    }

    #[test]
    fn workflow_deserialization_rejects_duplicate_step_ids() {
        let result = serde_json::from_str::<Workflow>(
            r#"{
                "name": "support",
                "steps": [
                    { "id": "classify", "prompt": "Classify" },
                    { "id": "classify", "prompt": "Classify again" }
                ]
            }"#,
        );

        assert_eq!(
            result.unwrap_err().to_string(),
            VogonError::DuplicateStepId("classify".to_owned()).to_string()
        );
    }

    #[test]
    fn workflow_deserialization_rejects_empty_step_prompts() {
        let result = serde_json::from_str::<Workflow>(
            r#"{
                "name": "support",
                "steps": [
                    { "id": "classify", "prompt": " " }
                ]
            }"#,
        );

        assert_eq!(
            result.unwrap_err().to_string(),
            VogonError::EmptyStepPrompt("classify".to_owned()).to_string()
        );
    }
}
