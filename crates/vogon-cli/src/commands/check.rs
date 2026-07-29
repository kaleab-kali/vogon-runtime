use std::path::Path;

use crate::commands::workflow_file::read_toml_workflow;

pub fn run(workflow_file: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let required_inputs = workflow.required_inputs()?;

    if json {
        let mut summary = serde_json::json!({
            "workflow_name": workflow.name(),
            "step_count": workflow.steps().len()
        });
        if !required_inputs.is_empty() {
            summary["required_inputs"] = serde_json::json!(required_inputs);
        }
        if let Some(decision) = workflow.decision() {
            summary["decision"] = serde_json::to_value(decision)?;
        }
        if let Some(execution) = workflow.execution() {
            summary["execution"] = serde_json::to_value(execution)?;
        }
        println!("{}", serde_json::to_string(&summary)?);
        return Ok(());
    }

    println!(
        "Workflow valid: {} ({} steps)",
        workflow.name(),
        workflow.steps().len()
    );
    if !required_inputs.is_empty() {
        println!("Required inputs: {}", required_inputs.join(", "));
    }
    if let Some(decision) = workflow.decision() {
        println!(
            "Decision gate: {} {} (allow: {}; deny: {})",
            decision.step.as_str(),
            decision.pointer,
            decision.allow.join(", "),
            decision.deny.join(", ")
        );
    }
    if let Some(execution) = workflow.execution() {
        println!(
            "Execution policy: providers={} models={} max_step_output_bytes={}",
            display_values(&execution.allowed_providers),
            display_values(&execution.allowed_models),
            execution
                .max_step_output_bytes
                .map_or_else(|| "unrestricted".to_owned(), |value| value.to_string())
        );
    }

    Ok(())
}

fn display_values(values: &[String]) -> String {
    if values.is_empty() {
        "unrestricted".to_owned()
    } else {
        values.join(", ")
    }
}
