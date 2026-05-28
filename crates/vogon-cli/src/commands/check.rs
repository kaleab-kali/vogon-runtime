use std::path::Path;

use crate::commands::workflow_file::read_toml_workflow;

pub fn run(workflow_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;

    println!(
        "Workflow valid: {} ({} steps)",
        workflow.name(),
        workflow.steps().len()
    );

    Ok(())
}
