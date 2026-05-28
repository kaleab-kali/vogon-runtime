use std::{path::PathBuf, process::Command};

#[test]
fn verify_command_accepts_support_triage_replay() {
    assert_verify_succeeds("support-triage");
}

#[test]
fn verify_command_accepts_writing_pipeline_replay() {
    assert_verify_succeeds("writing-pipeline");
}

fn assert_verify_succeeds(name: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path(name))
        .arg(replay_path(name))
        .output()
        .expect("verify command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("Replay verified: {name}")));
}

fn workflow_path(name: &str) -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join(format!("{name}.toml"))
}

fn replay_path(name: &str) -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("replays")
        .join(format!("{name}.replay.json"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}
