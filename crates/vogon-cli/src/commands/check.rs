use std::path::Path;

use crate::commands::workflow_file::read_toml_workflow;

pub fn run(workflow_file: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;

    if json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "workflow_name": workflow.name(),
                "step_count": workflow.steps().len()
            }))?
        );
        return Ok(());
    }

    println!(
        "Workflow valid: {} ({} steps)",
        workflow.name(),
        workflow.steps().len()
    );

    Ok(())
}
