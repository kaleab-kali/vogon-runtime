use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{Result, Step, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<Step>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Result<Self> {
        let workflow = Self {
            name: name.into(),
            steps,
        };
        workflow.validate()?;
        Ok(workflow)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(VogonError::EmptyWorkflowName);
        }

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
}
