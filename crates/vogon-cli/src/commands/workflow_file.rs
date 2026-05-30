use std::{io, path::Path};

use vogon_core::Workflow;

use crate::commands::file_io;

pub fn read_toml_workflow(path: &Path) -> Result<Workflow, Box<dyn std::error::Error>> {
    let workflow_text = file_io::read_to_string(path, "workflow file")?;
    let workflow: Workflow = toml::from_str(&workflow_text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse workflow file `{}`: {error}",
                path.display()
            ),
        )
    })?;
    workflow.validate()?;

    Ok(workflow)
}
