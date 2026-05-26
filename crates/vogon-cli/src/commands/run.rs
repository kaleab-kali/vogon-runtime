use std::path::Path;

use vogon_adapters::DeterministicEchoModel;
use vogon_core::Runtime;

use crate::commands::workflow_file::read_toml_workflow;

pub fn run(workflow_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let report = Runtime::new(DeterministicEchoModel).run(&workflow)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
