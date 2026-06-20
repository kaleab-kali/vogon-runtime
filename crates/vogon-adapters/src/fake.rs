use vogon_core::{ModelAdapter, Result, RuntimeMetadata, Step, stable_hash};

#[derive(Debug, Default, Clone, Copy)]
/// Deterministic adapter that echoes the step id and a stable hash of the input.
pub struct DeterministicEchoModel;

impl ModelAdapter for DeterministicEchoModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        Ok(format!("{}:{}", step.id().as_str(), stable_hash(input)))
    }

    fn cache_identity(&self) -> String {
        format!(
            "vogon-adapters@{}:deterministic-echo:v1",
            env!("CARGO_PKG_VERSION")
        )
    }

    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new(
            "deterministic",
            "deterministic-echo",
            env!("CARGO_PKG_VERSION"),
            self.cache_identity(),
        )
        .with_model("deterministic-echo")
        .with_parameter("mode", "offline")
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

    #[test]
    fn cache_identity_describes_deterministic_adapter() {
        let model = DeterministicEchoModel;

        assert!(model.cache_identity().contains("deterministic-echo"));
    }

    #[test]
    fn runtime_metadata_describes_deterministic_adapter() {
        let model = DeterministicEchoModel;

        let metadata = model.runtime_metadata();

        assert_eq!(metadata.provider, "deterministic");
        assert_eq!(metadata.adapter, "deterministic-echo");
        assert_eq!(metadata.model.as_deref(), Some("deterministic-echo"));
    }
}
