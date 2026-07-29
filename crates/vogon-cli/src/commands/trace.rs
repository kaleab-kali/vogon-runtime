use std::{io, path::Path};

use vogon_core::{RedactionSet, RunReport};

use crate::commands::file_io;
use crate::commands::redaction::parse_redactions;
use crate::commands::redaction_markers::validate_redaction_markers;

pub fn run(
    replay_file: &Path,
    jsonl: bool,
    redaction_values: &[String],
    redaction_environment_values: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let replay_text = file_io::read_to_string(replay_file, "replay file")?;
    let replay: RunReport = serde_json::from_str(&replay_text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse replay file `{}`: {error}",
                replay_file.display()
            ),
        )
    })?;
    validate_redaction_markers(&replay)?;
    let redactions = parse_redactions(redaction_values, redaction_environment_values)?;

    if jsonl {
        print_jsonl_trace(&replay, &redactions)?;
        return Ok(());
    }

    println!("Workflow: {}", replay.workflow_name);
    println!("Replay schema: {}", replay.schema_version);
    println!("Provider: {}", replay.runtime.provider);
    println!("Adapter: {}", replay.runtime.adapter);
    if let Some(model) = replay.runtime.model.as_deref() {
        println!("Model: {model}");
    }
    println!("Run hash: {}", replay.run_hash);
    println!("Steps: {}", replay.steps.len());

    for (index, step) in replay.steps.iter().enumerate() {
        println!();
        println!("[{}] {}", index + 1, step.step_id.as_str());
        println!("input_hash: {}", step.input_hash);
        println!("output_hash: {}", step.output_hash);
        println!("output: {}", redactions.redact(&step.output));
    }

    Ok(())
}

fn print_jsonl_trace(
    replay: &RunReport,
    redactions: &RedactionSet,
) -> Result<(), serde_json::Error> {
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "event": "run",
            "schema_version": replay.schema_version,
            "workflow_name": replay.workflow_name,
            "runtime": &replay.runtime,
            "run_hash": replay.run_hash,
            "step_count": replay.steps.len()
        }))?
    );

    for (index, step) in replay.steps.iter().enumerate() {
        let output = redactions.redact(&step.output);
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "event": "step",
                "index": index + 1,
                "step_id": step.step_id.as_str(),
                "input_hash": step.input_hash,
                "output_hash": step.output_hash,
                "output": output
            }))?
        );
    }

    Ok(())
}
