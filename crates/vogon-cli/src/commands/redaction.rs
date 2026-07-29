use std::{env, io};

use vogon_core::{RedactionRule, RedactionSet};

pub fn parse_redactions(
    values: &[String],
    environment_values: &[String],
) -> Result<RedactionSet, Box<dyn std::error::Error>> {
    let mut rules = Vec::with_capacity(values.len() + environment_values.len());
    for value in values {
        let (label, literal) = value.split_once('=').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("redaction `{value}` must use LABEL=VALUE"),
            )
        })?;
        rules.push(RedactionRule::new(label, literal)?);
    }

    for value in environment_values {
        let (label, variable_name) = value.split_once('=').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("environment redaction `{value}` must use LABEL=ENV_VAR"),
            )
        })?;
        if variable_name.is_empty()
            || !variable_name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "environment redaction `{value}` must name an ASCII alphanumeric or underscore environment variable"
                ),
            )
            .into());
        }

        let literal = match env::var(variable_name) {
            Ok(literal) => literal,
            Err(env::VarError::NotPresent) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("redaction environment variable `{variable_name}` is not set"),
                )
                .into());
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "redaction environment variable `{variable_name}` contains non-Unicode data"
                    ),
                )
                .into());
            }
        };
        rules.push(RedactionRule::new(label, literal)?);
    }

    Ok(RedactionSet::new(rules)?)
}
