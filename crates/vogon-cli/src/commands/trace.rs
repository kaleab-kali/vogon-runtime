use std::{fs, path::Path};

use vogon_core::RunReport;

pub fn run(replay_file: &Path, jsonl: bool) -> Result<(), Box<dyn std::error::Error>> {
    let replay_text = fs::read_to_string(replay_file)?;
    let replay: RunReport = serde_json::from_str(&replay_text)?;

    if jsonl {
        print_jsonl_trace(&replay)?;
        return Ok(());
    }

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

fn print_jsonl_trace(replay: &RunReport) -> Result<(), serde_json::Error> {
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "event": "run",
            "workflow_name": replay.workflow_name,
            "run_hash": replay.run_hash,
            "step_count": replay.steps.len()
        }))?
    );

    for (index, step) in replay.steps.iter().enumerate() {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "event": "step",
                "index": index + 1,
                "step_id": step.step_id.as_str(),
                "input_hash": step.input_hash,
                "output_hash": step.output_hash,
                "output": step.output
            }))?
        );
    }

    Ok(())
}
