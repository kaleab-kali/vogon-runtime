use vogon_adapters::DeterministicEchoModel;
use vogon_core::{Runtime, Step, StepId, Workflow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::new(
        "basic",
        vec![Step::new(
            StepId::new("classify")?,
            "Classify this input deterministically.",
        )],
    )?;

    let report = Runtime::new(DeterministicEchoModel).run(&workflow)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
