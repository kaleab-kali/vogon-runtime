use std::{collections::BTreeSet, io, path::Path};

use vogon_adapters::DeterministicEchoModel;
use vogon_core::{RedactionSet, ReplayMismatch, RunReport, Runtime, VerificationReport};

use crate::commands::file_io;
use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

const REDACTED_MISMATCH_OUTPUT: &str = "[UNREPORTED: replay is redacted]";

pub fn run(
    workflow_file: &Path,
    replay_file: &Path,
    redaction_values: &[String],
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

    if verification.is_match() {
        println!(
            "Replay verified: {} ({} steps)",
            replay.workflow_name,
            replay.steps.len()
        );
        return Ok(());
    }

    let mismatch_count = verification.mismatches.len();
    let printable_verification = if replay_redaction_labels.is_empty() {
        verification
    } else {
        mask_redacted_step_outputs(verification)
    };

    eprintln!("{}", serde_json::to_string_pretty(&printable_verification)?);
    Err(io::Error::other(format!(
        "replay verification failed with {mismatch_count} mismatch(es)"
    ))
    .into())
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

fn replay_redaction_labels(replay: &RunReport) -> io::Result<BTreeSet<String>> {
    let mut replay_labels = BTreeSet::new();
    for step in &replay.steps {
        collect_redaction_labels(&step.output, &mut replay_labels).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "replay step `{}` contains malformed redaction marker: {error}",
                    step.step_id.as_str()
                ),
            )
        })?;
    }

    Ok(replay_labels)
}

fn collect_redaction_labels(output: &str, labels: &mut BTreeSet<String>) -> io::Result<()> {
    const MARKER_PREFIX: &str = "[REDACTED:";

    let mut remaining = output;
    while let Some(start) = remaining.find(MARKER_PREFIX) {
        let after_prefix = &remaining[start + MARKER_PREFIX.len()..];
        let Some(end) = after_prefix.find(']') else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing closing `]`",
            ));
        };

        let label = &after_prefix[..end];
        if label.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty redaction label",
            ));
        }
        labels.insert(label.to_owned());

        remaining = &after_prefix[end + 1..];
    }

    Ok(())
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
