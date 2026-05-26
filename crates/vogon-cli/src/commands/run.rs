use std::{fs, path::Path};

use vogon_adapters::DeterministicEchoModel;
use vogon_core::{Runtime, Workflow};

pub fn run(workflow_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let workflow_text = fs::read_to_string(workflow_file)?;
    let workflow: Workflow = toml::from_str(&workflow_text)?;
    workflow.validate()?;

    let report = Runtime::new(DeterministicEchoModel).run(&workflow)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
