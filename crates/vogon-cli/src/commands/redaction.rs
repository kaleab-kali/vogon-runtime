use std::{collections::BTreeSet, io};

use vogon_core::{RedactionRule, RedactionSet};

pub fn parse_redactions(values: &[String]) -> Result<RedactionSet, Box<dyn std::error::Error>> {
    let mut labels = BTreeSet::new();
    let rules = values
        .iter()
        .map(|value| {
            let (label, literal) = value.split_once('=').ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("redaction `{value}` must use LABEL=VALUE"),
                )
            })?;
            if !labels.insert(label.to_owned()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("redaction label `{label}` is configured more than once"),
                )
                .into());
            }

            Ok(RedactionRule::new(label, literal)?)
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok(RedactionSet::new(rules))
}
