use std::path::Path;

pub fn run(replay_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "trace command scaffolded for replay file: {}",
        replay_file.display()
    );
    Ok(())
}
