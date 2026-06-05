use std::{collections::BTreeSet, io};

use vogon_core::{RedactionRule, RunReport};

pub fn validate_redaction_markers(replay: &RunReport) -> io::Result<()> {
    replay_redaction_labels(replay).map(|_| ())
}

pub fn replay_redaction_labels(replay: &RunReport) -> io::Result<BTreeSet<String>> {
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

        validate_redaction_label(label)?;
        labels.insert(label.to_owned());

        remaining = &after_prefix[end + 1..];
    }

    Ok(())
}

fn validate_redaction_label(label: &str) -> io::Result<()> {
    RedactionRule::new(label, "marker-validation")
        .map(|_| ())
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid redaction label `{label}`: {error}"),
            )
        })
}
