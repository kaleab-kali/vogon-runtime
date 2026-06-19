use vogon_core::{ModelAdapter, Result, Step, stable_hash};

#[derive(Debug, Default, Clone, Copy)]
/// Deterministic adapter that echoes the step id and a stable hash of the input.
pub struct DeterministicEchoModel;

impl ModelAdapter for DeterministicEchoModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        Ok(format!("{}:{}", step.id().as_str(), stable_hash(input)))
    }
}

#[cfg(test)]
mod tests {
    use vogon_core::{ModelAdapter, Step, StepId};

    use super::DeterministicEchoModel;

    #[test]
    fn echo_model_is_deterministic() {
        let model = DeterministicEchoModel;
        let step = Step::new(StepId::new("classify").unwrap(), "Classify input");

        assert_eq!(
            model.complete(&step, "same input").unwrap(),
            model.complete(&step, "same input").unwrap()
        );
    }
}
