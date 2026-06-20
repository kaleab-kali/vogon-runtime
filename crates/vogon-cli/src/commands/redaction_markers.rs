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

    let mut search_start = 0;
    while let Some(relative_start) = output[search_start..].find(MARKER_PREFIX) {
        let marker_start = search_start + relative_start;
        let after_prefix_start = marker_start + MARKER_PREFIX.len();

        if is_escaped_marker(output, marker_start) {
            search_start = after_prefix_start;
            continue;
        }

        let after_prefix = &output[after_prefix_start..];
        if let Some(next_marker) = after_prefix.find(MARKER_PREFIX) {
            if after_prefix.find(']').is_none_or(|end| next_marker < end) {
                search_start = after_prefix_start + next_marker;
                continue;
            }
        }

        let Some(end) = after_prefix.find(']') else {
            if !is_unclosed_marker_candidate(after_prefix) {
                search_start = after_prefix_start;
                continue;
            }

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

        search_start = after_prefix_start + end + 1;
    }

    Ok(())
}

fn is_escaped_marker(output: &str, marker_start: usize) -> bool {
    let preceding_backslashes = output.as_bytes()[..marker_start]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count();

    preceding_backslashes % 2 == 1
}

fn is_unclosed_marker_candidate(after_prefix: &str) -> bool {
    after_prefix.is_empty() || after_prefix.chars().all(is_redaction_label_char)
}

fn is_redaction_label_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
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

#[cfg(test)]
mod tests {
    use super::collect_redaction_labels;
    use std::{collections::BTreeSet, io};

    fn labels_for(output: &str) -> io::Result<BTreeSet<String>> {
        let mut labels = BTreeSet::new();
        collect_redaction_labels(output, &mut labels)?;
        Ok(labels)
    }

    #[test]
    fn collects_valid_redaction_markers() {
        let labels = labels_for("token=[REDACTED:api_key] secret=[REDACTED:session-id]")
            .expect("valid markers should parse");

        assert_eq!(
            labels,
            BTreeSet::from(["api_key".to_owned(), "session-id".to_owned()])
        );
    }

    #[test]
    fn ignores_escaped_redaction_marker_text() {
        let labels =
            labels_for(r"literal \[REDACTED:api_key]").expect("escaped marker should be literal");

        assert!(labels.is_empty());
    }

    #[test]
    fn ignores_unclosed_marker_like_prose() {
        let labels = labels_for("docs mention [REDACTED:api_key in examples")
            .expect("marker-like prose should not be malformed");

        assert!(labels.is_empty());
    }

    #[test]
    fn does_not_merge_marker_like_text_across_later_marker() {
        let labels = labels_for("docs mention [REDACTED:api_key in prose [REDACTED:real]")
            .expect("later valid marker should parse independently");

        assert_eq!(labels, BTreeSet::from(["real".to_owned()]));
    }

    #[test]
    fn rejects_unclosed_marker_with_valid_label_chars() {
        let error = labels_for("[REDACTED:api_key").unwrap_err().to_string();

        assert!(error.contains("missing closing `]`"));
    }

    #[test]
    fn rejects_closed_marker_with_invalid_label() {
        let error = labels_for("[REDACTED:bad label]").unwrap_err().to_string();

        assert!(error.contains("invalid redaction label `bad label`"));
    }

    #[test]
    fn rejects_empty_marker_label() {
        let error = labels_for("[REDACTED:]").unwrap_err().to_string();

        assert!(error.contains("empty redaction label"));
    }
}
