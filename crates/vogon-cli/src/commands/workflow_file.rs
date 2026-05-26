use std::{fs, path::Path};

use vogon_core::Workflow;

pub fn read_toml_workflow(path: &Path) -> Result<Workflow, Box<dyn std::error::Error>> {
    let workflow_text = fs::read_to_string(path)?;
    let workflow: Workflow = toml::from_str(&workflow_text)?;
    workflow.validate()?;

    Ok(workflow)
}
