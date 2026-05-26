use vogon_adapters::DeterministicEchoModel;
use vogon_core::{Runtime, Step, StepId, Workflow};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = Workflow::new(
        "demo",
        vec![
            Step::new(
                StepId::new("classify")?,
                "Classify this support request as billing, bug, or general.",
            ),
            Step::new(
                StepId::new("draft_response")?,
                "Draft a concise customer response.",
            ),
        ],
    )?;

    let report = Runtime::new(DeterministicEchoModel).run(&workflow)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
