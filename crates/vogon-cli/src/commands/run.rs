use std::path::Path;

use vogon_adapters::DeterministicEchoModel;
use vogon_core::Runtime;

use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

pub fn run(
    workflow_file: &Path,
    redaction_values: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let redactions = parse_redactions(redaction_values)?;
    let report =
        Runtime::new(DeterministicEchoModel).run_with_redactions(&workflow, &redactions)?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
