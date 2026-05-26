use std::{path::PathBuf, process::Command};

#[test]
fn trace_command_prints_replay_summary() {
    let replay = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("replays")
        .join("support-triage.replay.json");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(replay)
        .output()
        .expect("trace command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workflow: support-triage"));
    assert!(stdout.contains("[1] classify"));
    assert!(stdout.contains("[2] draft_response"));
}
