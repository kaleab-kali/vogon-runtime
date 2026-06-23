use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use vogon_core::Workflow;

const STARTER_WORKFLOW: &str = r#"name = "starter-workflow"

[[steps]]
id = "draft"
prompt = """
Write a concise project update from the notes below.

Notes:
- Replace this text with your own input.
"""

[[steps]]
id = "review"
prompt = """
Review the draft for clarity, missing context, and next actions.
"""
"#;

pub fn run(output: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    validate_starter_workflow()?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create parent directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
    }

    if force {
        fs::write(output, STARTER_WORKFLOW).map_err(|error| {
            format!(
                "failed to write workflow file `{}`: {error}",
                output.display()
            )
        })?;
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    format!(
                        "workflow file `{}` already exists; pass --force to overwrite",
                        output.display()
                    )
                } else {
                    format!(
                        "failed to create workflow file `{}`: {error}",
                        output.display()
                    )
                }
            })?;
        file.write_all(STARTER_WORKFLOW.as_bytes())
            .map_err(|error| {
                format!(
                    "failed to write workflow file `{}`: {error}",
                    output.display()
                )
            })?;
    }

    println!("Created workflow file: {}", output.display());
    println!("Next: vogon check {}", output.display());
    Ok(())
}

fn validate_starter_workflow() -> Result<(), Box<dyn std::error::Error>> {
    toml::from_str::<Workflow>(STARTER_WORKFLOW)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::STARTER_WORKFLOW;

    #[test]
    fn starter_workflow_is_valid_toml() {
        toml::from_str::<vogon_core::Workflow>(STARTER_WORKFLOW).unwrap();
    }
}
