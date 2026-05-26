use std::{fs, path::Path};

use vogon_adapters::DeterministicEchoModel;
use vogon_core::Runtime;

use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

pub fn run(
    workflow_file: &Path,
    redaction_values: &[String],
    output: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let redactions = parse_redactions(redaction_values)?;
    let report =
        Runtime::new(DeterministicEchoModel).run_with_redactions(&workflow, &redactions)?;
    let replay_json = format!("{}\n", serde_json::to_string_pretty(&report)?);

    if let Some(output) = output {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }

        fs::write(output, replay_json)?;
        println!("Replay written: {}", output.display());
    } else {
        print!("{replay_json}");
    }

    Ok(())
}
