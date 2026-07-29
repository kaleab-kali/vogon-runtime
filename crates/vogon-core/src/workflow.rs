use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{DecisionPolicy, ExecutionPolicy, Result, Step, VogonError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
/// Ordered collection of workflow steps.
pub struct Workflow {
    /// Stable workflow name.
    pub name: String,
    /// Ordered workflow steps.
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional machine-enforceable decision policy for the final step.
    pub decision: Option<DecisionPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional provider, model, and output restrictions.
    pub execution: Option<ExecutionPolicy>,
    #[serde(skip)]
    inputs_rendered: bool,
}

impl Workflow {
    /// Creates and validates a workflow.
    pub fn new(name: impl Into<String>, steps: Vec<Step>) -> Result<Self> {
        let workflow = Self {
            name: name.into(),
            steps,
            decision: None,
            execution: None,
            inputs_rendered: false,
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

    /// Returns the optional final-step decision policy.
    pub fn decision(&self) -> Option<&DecisionPolicy> {
        self.decision.as_ref()
    }

    /// Returns the optional execution policy.
    pub fn execution(&self) -> Option<&ExecutionPolicy> {
        self.execution.as_ref()
    }

    /// Attaches and validates a final-step decision policy.
    pub fn with_decision(mut self, decision: DecisionPolicy) -> Result<Self> {
        self.decision = Some(decision);
        self.validate()?;
        Ok(self)
    }

    /// Attaches and validates provider, model, and output restrictions.
    pub fn with_execution_policy(mut self, execution: ExecutionPolicy) -> Result<Self> {
        self.execution = Some(execution);
        self.validate()?;
        Ok(self)
    }

    /// Returns the sorted names of inputs referenced by step prompt placeholders.
    ///
    /// Workflow inputs use the exact `{{input.NAME}}` syntax. Names may contain
    /// ASCII letters, digits, underscores, and hyphens.
    pub fn required_inputs(&self) -> Result<Vec<String>> {
        if self.inputs_rendered {
            return Ok(Vec::new());
        }

        let mut names = BTreeSet::new();
        for step in &self.steps {
            for placeholder in input_placeholders(step)? {
                names.insert(placeholder.name);
            }
        }
        Ok(names.into_iter().collect())
    }

    /// Replaces workflow input placeholders with supplied values.
    ///
    /// Rendering is strict: missing inputs, unused inputs, malformed
    /// placeholders, and invalid input names are rejected.
    pub fn render_inputs(&self, inputs: &BTreeMap<String, String>) -> Result<Self> {
        let required = self.required_inputs()?.into_iter().collect::<BTreeSet<_>>();

        for name in inputs.keys() {
            validate_workflow_input_name(name)?;
            if !required.contains(name) {
                return Err(VogonError::UnusedWorkflowInput(name.clone()));
            }
        }

        for name in &required {
            if !inputs.contains_key(name) {
                return Err(VogonError::MissingWorkflowInput(name.clone()));
            }
        }

        let steps = self
            .steps
            .iter()
            .map(|step| {
                Ok(Step::new(
                    step.id().clone(),
                    render_step_prompt(step, inputs)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        let rendered = Workflow {
            name: self.name.clone(),
            steps,
            decision: self.decision.clone(),
            execution: self.execution.clone(),
            inputs_rendered: true,
        };
        rendered.validate()?;
        Ok(rendered)
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

            if !self.inputs_rendered {
                input_placeholders(step)?;
            }
        }

        if let Some(decision) = &self.decision {
            decision.validate(self.steps.last().expect("steps are non-empty").id())?;
        }
        if let Some(execution) = &self.execution {
            execution.validate()?;
        }

        Ok(())
    }
}

const INPUT_PLACEHOLDER_PREFIX: &str = "{{input.";
const INPUT_PLACEHOLDER_SUFFIX: &str = "}}";

#[derive(Debug)]
struct InputPlaceholder {
    start: usize,
    end: usize,
    name: String,
}

fn input_placeholders(step: &Step) -> Result<Vec<InputPlaceholder>> {
    let prompt = step.prompt();
    let mut placeholders = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = prompt[cursor..].find(INPUT_PLACEHOLDER_PREFIX) {
        let start = cursor + relative_start;
        let name_start = start + INPUT_PLACEHOLDER_PREFIX.len();
        let Some(relative_end) = prompt[name_start..].find(INPUT_PLACEHOLDER_SUFFIX) else {
            return Err(VogonError::MalformedWorkflowInputPlaceholder(
                step.id().as_str().to_owned(),
                prompt[start..].to_owned(),
            ));
        };
        let name_end = name_start + relative_end;
        let name = &prompt[name_start..name_end];
        validate_workflow_input_name(name)?;
        let end = name_end + INPUT_PLACEHOLDER_SUFFIX.len();

        placeholders.push(InputPlaceholder {
            start,
            end,
            name: name.to_owned(),
        });
        cursor = end;
    }

    Ok(placeholders)
}

fn validate_workflow_input_name(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(VogonError::InvalidWorkflowInputName(name.to_owned()));
    }
    Ok(())
}

fn render_step_prompt(step: &Step, inputs: &BTreeMap<String, String>) -> Result<String> {
    let prompt = step.prompt();
    let placeholders = input_placeholders(step)?;
    let inserted_bytes = placeholders
        .iter()
        .map(|placeholder| inputs[&placeholder.name].len())
        .sum::<usize>();
    let placeholder_bytes = placeholders
        .iter()
        .map(|placeholder| placeholder.end - placeholder.start)
        .sum::<usize>();
    let mut rendered = String::with_capacity(prompt.len() - placeholder_bytes + inserted_bytes);
    let mut cursor = 0;

    for placeholder in placeholders {
        rendered.push_str(&prompt[cursor..placeholder.start]);
        rendered.push_str(&inputs[&placeholder.name]);
        cursor = placeholder.end;
    }
    rendered.push_str(&prompt[cursor..]);

    Ok(rendered)
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
            #[serde(default)]
            decision: Option<DecisionPolicy>,
            #[serde(default)]
            execution: Option<ExecutionPolicy>,
        }

        let fields = WorkflowFields::deserialize(deserializer)?;
        let workflow = Workflow {
            name: fields.name,
            steps: fields.steps,
            decision: fields.decision,
            execution: fields.execution,
            inputs_rendered: false,
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
    use std::collections::BTreeMap;

    use crate::{DecisionPolicy, Step, StepId, VogonError, Workflow};

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

    #[test]
    fn workflow_reports_sorted_unique_required_inputs() {
        let workflow = Workflow::new(
            "review",
            vec![
                Step::new(
                    StepId::new("inspect").unwrap(),
                    "Review {{input.git_diff}} for {{input.service}}.",
                ),
                Step::new(
                    StepId::new("decide").unwrap(),
                    "Decide for {{input.service}}.",
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            workflow.required_inputs().unwrap(),
            vec!["git_diff".to_owned(), "service".to_owned()]
        );
    }

    #[test]
    fn workflow_renders_repeated_inputs_without_recursive_expansion() {
        let workflow = Workflow::new(
            "review",
            vec![Step::new(
                StepId::new("inspect").unwrap(),
                "Review {{input.git_diff}} for {{input.service}} and {{input.service}}.",
            )],
        )
        .unwrap();
        let inputs = BTreeMap::from([
            (
                "git_diff".to_owned(),
                "change includes {{input.service}}".to_owned(),
            ),
            ("service".to_owned(), "payments".to_owned()),
        ]);

        let rendered = workflow.render_inputs(&inputs).unwrap();

        assert_eq!(
            rendered.steps()[0].prompt(),
            "Review change includes {{input.service}} for payments and payments."
        );
        assert!(rendered.required_inputs().unwrap().is_empty());
    }

    #[test]
    fn workflow_rejects_missing_inputs() {
        let workflow = Workflow::new(
            "review",
            vec![Step::new(
                StepId::new("inspect").unwrap(),
                "Review {{input.git_diff}}.",
            )],
        )
        .unwrap();

        assert_eq!(
            workflow.render_inputs(&BTreeMap::new()).unwrap_err(),
            VogonError::MissingWorkflowInput("git_diff".to_owned())
        );
    }

    #[test]
    fn workflow_rejects_unused_inputs() {
        let workflow = Workflow::new(
            "review",
            vec![Step::new(StepId::new("inspect").unwrap(), "Review this.")],
        )
        .unwrap();
        let inputs = BTreeMap::from([("git_diff".to_owned(), "diff".to_owned())]);

        assert_eq!(
            workflow.render_inputs(&inputs).unwrap_err(),
            VogonError::UnusedWorkflowInput("git_diff".to_owned())
        );
    }

    #[test]
    fn workflow_rejects_malformed_input_placeholders() {
        let result = Workflow::new(
            "review",
            vec![Step::new(
                StepId::new("inspect").unwrap(),
                "Review {{input.git_diff}.",
            )],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::MalformedWorkflowInputPlaceholder(
                "inspect".to_owned(),
                "{{input.git_diff}.".to_owned()
            )
        );
    }

    #[test]
    fn workflow_rejects_invalid_input_names() {
        let result = Workflow::new(
            "review",
            vec![Step::new(
                StepId::new("inspect").unwrap(),
                "Review {{input.git diff}}.",
            )],
        );

        assert_eq!(
            result.unwrap_err(),
            VogonError::InvalidWorkflowInputName("git diff".to_owned())
        );
    }

    #[test]
    fn workflow_accepts_a_final_step_decision_policy() {
        let workflow = Workflow::new(
            "release",
            vec![
                Step::new(StepId::new("review").unwrap(), "Review"),
                Step::new(StepId::new("decide").unwrap(), "Decide"),
            ],
        )
        .unwrap()
        .with_decision(DecisionPolicy {
            step: StepId::new("decide").unwrap(),
            pointer: "/decision".to_owned(),
            allow: vec!["GO".to_owned()],
            deny: vec!["NO_GO".to_owned()],
        })
        .unwrap();

        assert_eq!(workflow.decision().unwrap().step.as_str(), "decide");
    }

    #[test]
    fn workflow_deserialization_rejects_a_nonfinal_decision_step() {
        let error = serde_json::from_str::<Workflow>(
            r#"{
                "name": "release",
                "decision": {
                    "step": "review",
                    "pointer": "/decision",
                    "allow": ["GO"],
                    "deny": ["NO_GO"]
                },
                "steps": [
                    {"id": "review", "prompt": "Review"},
                    {"id": "decide", "prompt": "Decide"}
                ]
            }"#,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("must be the final workflow step")
        );
    }
}
