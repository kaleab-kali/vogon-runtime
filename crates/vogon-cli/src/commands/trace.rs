use std::{fs, path::Path};

use vogon_core::RunReport;

pub fn run(replay_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let replay_text = fs::read_to_string(replay_file)?;
    let replay: RunReport = serde_json::from_str(&replay_text)?;

    println!("Workflow: {}", replay.workflow_name);
    println!("Run hash: {}", replay.run_hash);
    println!("Steps: {}", replay.steps.len());

    for (index, step) in replay.steps.iter().enumerate() {
        println!();
        println!("[{}] {}", index + 1, step.step_id.as_str());
        println!("input_hash: {}", step.input_hash);
        println!("output_hash: {}", step.output_hash);
        println!("output: {}", step.output);
    }

    Ok(())
}
