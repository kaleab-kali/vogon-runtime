use std::path::Path;

pub fn run(workflow_file: &Path, replay_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "verify command scaffolded for workflow file `{}` and replay file `{}`",
        workflow_file.display(),
        replay_file.display()
    );
    Ok(())
}
