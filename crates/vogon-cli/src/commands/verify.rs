use std::{fs, io, path::Path};

use vogon_adapters::DeterministicEchoModel;
use vogon_core::{RunReport, Runtime};

use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

pub fn run(
    workflow_file: &Path,
    replay_file: &Path,
    redaction_values: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let replay_text = fs::read_to_string(replay_file)?;
    let replay: RunReport = serde_json::from_str(&replay_text)?;
    let redactions = parse_redactions(redaction_values)?;
    let verification = Runtime::new(DeterministicEchoModel).verify_with_redactions(
        &workflow,
        &replay,
        &redactions,
    )?;

    if verification.is_match() {
        println!(
            "Replay verified: {} ({} steps)",
            replay.workflow_name,
            replay.steps.len()
        );
        return Ok(());
    }

    eprintln!("{}", serde_json::to_string_pretty(&verification)?);
    Err(io::Error::other(format!(
        "replay verification failed with {} mismatch(es)",
        verification.mismatches.len()
    ))
    .into())
}
