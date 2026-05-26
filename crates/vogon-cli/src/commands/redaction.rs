use std::io;

use vogon_core::{RedactionRule, RedactionSet};

pub fn parse_redactions(values: &[String]) -> Result<RedactionSet, Box<dyn std::error::Error>> {
    let rules = values
        .iter()
        .map(|value| {
            let (label, literal) = value.split_once('=').ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("redaction `{value}` must use LABEL=VALUE"),
                )
            })?;

            Ok(RedactionRule::new(label, literal)?)
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    Ok(RedactionSet::new(rules))
}
