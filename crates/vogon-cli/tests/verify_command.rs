use std::{path::PathBuf, process::Command};

#[test]
fn verify_command_accepts_matching_replay() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let workflow = repo_root
        .join("fixtures")
        .join("workflows")
        .join("support-triage.toml");
    let replay = repo_root
        .join("fixtures")
        .join("replays")
        .join("support-triage.replay.json");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow)
        .arg(replay)
        .output()
        .expect("verify command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Replay verified: support-triage"));
}
