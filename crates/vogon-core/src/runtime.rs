use crate::{Result, RunReport, Step, StepResult, Workflow, stable_hash};

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
        workflow.validate()?;

        let mut previous_output = String::new();
        let mut steps = Vec::with_capacity(workflow.steps().len());

        for step in workflow.steps() {
            let input = step_input(step, &previous_output);
            let output = self.adapter.complete(step, &input)?;

            steps.push(StepResult {
                step_id: step.id().clone(),
                input_hash: stable_hash(&input),
                output_hash: stable_hash(&output),
                output: output.clone(),
            });

            previous_output = output;
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
    use crate::{Result, Step, StepId, Workflow};

    use super::{ModelAdapter, Runtime};

    #[derive(Debug, Clone)]
    struct TestModel;

    impl ModelAdapter for TestModel {
        fn complete(&self, step: &Step, input: &str) -> Result<String> {
            Ok(format!("{}:{input}", step.id().as_str()))
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
}
