use crate::{
    RedactionSet, ReplayMismatch, Result, RunReport, RuntimeEvent, Step, StepResult,
    VerificationReport, Workflow, stable_hash,
};

pub trait ModelAdapter {
    fn complete(&self, step: &Step, input: &str) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct Runtime<A> {
    adapter: A,
}

impl<A> Runtime<A>
where
    A: ModelAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn run(&self, workflow: &Workflow) -> Result<RunReport> {
        self.run_with_redactions_and_observer(workflow, &RedactionSet::empty(), |_| {})
    }

    pub fn run_with_observer<F>(&self, workflow: &Workflow, mut observer: F) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_with_redactions_and_observer(workflow, &RedactionSet::empty(), &mut observer)
    }

    pub fn run_with_redactions(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
    ) -> Result<RunReport> {
        self.run_with_redactions_and_observer(workflow, redactions, |_| {})
    }

    pub fn run_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
        mut observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        workflow.validate()?;

        let mut previous_output = String::new();
        let mut steps = Vec::with_capacity(workflow.steps().len());

        for step in workflow.steps() {
            observer(RuntimeEvent::StepStarted {
                step_id: step.id().clone(),
            });

            let input = step_input(step, &previous_output);
            let output = self.adapter.complete(step, &input)?;
            let redacted_output = redactions.redact(&output);

            steps.push(StepResult {
                step_id: step.id().clone(),
                input_hash: stable_hash(&input),
                output_hash: stable_hash(&redacted_output),
                output: redacted_output,
            });

            previous_output = output;

            observer(RuntimeEvent::StepFinished {
                step_id: step.id().clone(),
            });
        }

        let run_hash_material = steps
            .iter()
            .map(|step| {
                format!(
                    "{}:{}:{}",
                    step.step_id.as_str(),
                    step.input_hash,
                    step.output_hash
                )
            })
            .collect::<Vec<_>>()
            .join("|");

        Ok(RunReport {
            workflow_name: workflow.name().to_owned(),
            run_hash: stable_hash(run_hash_material),
            steps,
        })
    }

    pub fn verify(&self, workflow: &Workflow, expected: &RunReport) -> Result<VerificationReport> {
        self.verify_with_redactions_and_observer(workflow, expected, &RedactionSet::empty(), |_| {})
    }

    pub fn verify_with_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.verify_with_redactions_and_observer(
            workflow,
            expected,
            &RedactionSet::empty(),
            &mut observer,
        )
    }

    pub fn verify_with_redactions(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        redactions: &RedactionSet,
    ) -> Result<VerificationReport> {
        self.verify_with_redactions_and_observer(workflow, expected, redactions, |_| {})
    }

    pub fn verify_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        redactions: &RedactionSet,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        let actual = self.run_with_redactions_and_observer(workflow, redactions, &mut observer)?;
        let mut mismatches = Vec::new();

        if expected.workflow_name != actual.workflow_name {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::WorkflowName {
                    expected: expected.workflow_name.clone(),
                    actual: actual.workflow_name.clone(),
                },
                &mut observer,
            );
        }

        if expected.run_hash != actual.run_hash {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::RunHash {
                    expected: expected.run_hash.clone(),
                    actual: actual.run_hash.clone(),
                },
                &mut observer,
            );
        }

        if expected.steps.len() != actual.steps.len() {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::StepCount {
                    expected: expected.steps.len(),
                    actual: actual.steps.len(),
                },
                &mut observer,
            );
        }

        for (index, (expected_step, actual_step)) in
            expected.steps.iter().zip(actual.steps.iter()).enumerate()
        {
            if expected_step.step_id != actual_step.step_id {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepId {
                        index,
                        expected: expected_step.step_id.clone(),
                        actual: actual_step.step_id.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.input_hash != actual_step.input_hash {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepInputHash {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.input_hash.clone(),
                        actual: actual_step.input_hash.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.output_hash != actual_step.output_hash {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepOutputHash {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.output_hash.clone(),
                        actual: actual_step.output_hash.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.output != actual_step.output {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepOutput {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.output.clone(),
                        actual: actual_step.output.clone(),
                    },
                    &mut observer,
                );
            }
        }

        Ok(VerificationReport {
            workflow_name: actual.workflow_name,
            mismatches,
        })
    }
}

fn push_mismatch<F>(
    mismatches: &mut Vec<ReplayMismatch>,
    mismatch: ReplayMismatch,
    observer: &mut F,
) where
    F: FnMut(RuntimeEvent),
{
    observer(RuntimeEvent::ReplayMismatch {
        step_id: mismatch.step_id().cloned(),
    });
    mismatches.push(mismatch);
}

fn step_input(step: &Step, previous_output: &str) -> String {
    if previous_output.is_empty() {
        step.prompt().to_owned()
    } else {
        format!("{}\n\nPrevious output:\n{}", step.prompt(), previous_output)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RedactionRule, RedactionSet, Result, RuntimeEvent, Step, StepId, Workflow, stable_hash,
    };

    use super::{ModelAdapter, Runtime};

    #[derive(Debug, Clone)]
    struct TestModel;

    impl ModelAdapter for TestModel {
        fn complete(&self, step: &Step, input: &str) -> Result<String> {
            Ok(format!("{}:{input}", step.id().as_str()))
        }
    }

    #[derive(Debug, Clone)]
    struct SecretModel;

    impl ModelAdapter for SecretModel {
        fn complete(&self, _step: &Step, _input: &str) -> Result<String> {
            Ok("token=sk-test-123".to_owned())
        }
    }

    #[test]
    fn runtime_produces_a_report_for_ordered_steps() {
        let workflow = Workflow::new(
            "demo",
            vec![
                Step::new(StepId::new("first").unwrap(), "hello"),
                Step::new(StepId::new("second").unwrap(), "world"),
            ],
        )
        .unwrap();

        let report = Runtime::new(TestModel).run(&workflow).unwrap();

        assert_eq!(report.workflow_name, "demo");
        assert_eq!(report.steps.len(), 2);
        assert_ne!(report.run_hash, "");
    }

    #[test]
    fn run_with_observer_emits_step_events() {
        let workflow = Workflow::new(
            "demo",
            vec![
                Step::new(StepId::new("first").unwrap(), "hello"),
                Step::new(StepId::new("second").unwrap(), "world"),
            ],
        )
        .unwrap();
        let mut events = Vec::new();

        Runtime::new(TestModel)
            .run_with_observer(&workflow, |event| events.push(event))
            .unwrap();

        assert_eq!(
            events,
            vec![
                RuntimeEvent::StepStarted {
                    step_id: StepId::new("first").unwrap()
                },
                RuntimeEvent::StepFinished {
                    step_id: StepId::new("first").unwrap()
                },
                RuntimeEvent::StepStarted {
                    step_id: StepId::new("second").unwrap()
                },
                RuntimeEvent::StepFinished {
                    step_id: StepId::new("second").unwrap()
                },
            ]
        );
    }

    #[test]
    fn verify_accepts_matching_replay() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let replay = runtime.run(&workflow).unwrap();

        let verification = runtime.verify(&workflow, &replay).unwrap();

        assert!(verification.is_match());
    }

    #[test]
    fn verify_reports_replay_mismatch() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let mut replay = runtime.run(&workflow).unwrap();
        replay.steps[0].output = "changed".to_owned();

        let verification = runtime.verify(&workflow, &replay).unwrap();

        assert!(!verification.is_match());
    }

    #[test]
    fn verify_with_observer_emits_replay_mismatch_events() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let mut replay = runtime.run(&workflow).unwrap();
        replay.steps[0].output = "changed".to_owned();
        let mut events = Vec::new();

        runtime
            .verify_with_observer(&workflow, &replay, |event| events.push(event))
            .unwrap();

        assert!(events.contains(&RuntimeEvent::ReplayMismatch {
            step_id: Some(StepId::new("first").unwrap())
        }));
    }

    #[test]
    fn run_with_redactions_scrubs_step_outputs() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]);

        let report = Runtime::new(SecretModel)
            .run_with_redactions(&workflow, &redactions)
            .unwrap();

        assert_eq!(report.steps[0].output, "token=[REDACTED:api_key]");
        assert_eq!(
            report.steps[0].output_hash,
            stable_hash("token=[REDACTED:api_key]")
        );
    }

    #[test]
    fn verify_with_redactions_accepts_redacted_replay() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(SecretModel);
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]);
        let replay = runtime.run_with_redactions(&workflow, &redactions).unwrap();

        let verification = runtime
            .verify_with_redactions(&workflow, &replay, &redactions)
            .unwrap();

        assert!(verification.is_match());
    }
}
