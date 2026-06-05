use std::{collections::BTreeSet, io, path::Path};

use serde_json::json;
use vogon_adapters::DeterministicEchoModel;
use vogon_core::{RedactionSet, ReplayMismatch, RunReport, Runtime, VerificationReport};

use crate::commands::file_io;
use crate::commands::redaction::parse_redactions;
use crate::commands::redaction_markers::replay_redaction_labels;
use crate::commands::workflow_file::read_toml_workflow;

const REDACTED_MISMATCH_OUTPUT: &str = "[UNREPORTED: replay is redacted]";

pub fn run(
    workflow_file: &Path,
    replay_file: &Path,
    redaction_values: &[String],
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
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
    let redactions = parse_redactions(redaction_values)?;
    let replay_redaction_labels = replay_redaction_labels(&replay)?;
    let missing_redaction_labels =
        missing_replay_redaction_labels(&replay_redaction_labels, &redactions);
    if !missing_redaction_labels.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "replay contains redaction marker(s) without matching --redact label(s): {}",
                missing_redaction_labels.join(", ")
            ),
        )
        .into());
    }

    let verification = Runtime::new(DeterministicEchoModel).verify_with_redactions(
        &workflow,
        &replay,
        &redactions,
    )?;

    let mismatch_count = verification.mismatches.len();
    let printable_verification = if replay_redaction_labels.is_empty() {
        verification
    } else {
        mask_redacted_step_outputs(verification)
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_verification_report(&printable_verification))?
        );

        if printable_verification.is_match() {
            return Ok(());
        }

        return Err(io::Error::other(format!(
            "replay verification failed with {mismatch_count} mismatch(es)"
        ))
        .into());
    }

    if printable_verification.is_match() {
        println!(
            "Replay verified: {} ({} steps)",
            replay.workflow_name,
            replay.steps.len()
        );
        return Ok(());
    }

    eprintln!("{}", serde_json::to_string_pretty(&printable_verification)?);
    Err(io::Error::other(format!(
        "replay verification failed with {mismatch_count} mismatch(es)"
    ))
    .into())
}

fn json_verification_report(report: &VerificationReport) -> serde_json::Value {
    json!({
        "workflow_name": report.workflow_name,
        "is_match": report.is_match(),
        "mismatches": report.mismatches,
    })
}

fn missing_replay_redaction_labels(
    replay_labels: &BTreeSet<String>,
    redactions: &RedactionSet,
) -> Vec<String> {
    let configured_labels = redactions
        .rules()
        .iter()
        .map(|rule| rule.label.as_str())
        .collect::<BTreeSet<_>>();

    replay_labels
        .iter()
        .filter(|label| !configured_labels.contains(label.as_str()))
        .cloned()
        .collect()
}

fn mask_redacted_step_outputs(report: VerificationReport) -> VerificationReport {
    VerificationReport {
        workflow_name: report.workflow_name,
        mismatches: report
            .mismatches
            .into_iter()
            .map(mask_redacted_step_output)
            .collect(),
    }
}

fn mask_redacted_step_output(mismatch: ReplayMismatch) -> ReplayMismatch {
    match mismatch {
        ReplayMismatch::StepOutput {
            step_id, expected, ..
        } => ReplayMismatch::StepOutput {
            step_id,
            expected,
            actual: REDACTED_MISMATCH_OUTPUT.to_owned(),
        },
        other => other,
    }
}
