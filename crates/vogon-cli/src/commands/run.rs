use std::path::Path;

pub fn run(workflow_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "run command scaffolded for workflow file: {}",
        workflow_file.display()
    );
    Ok(())
}
