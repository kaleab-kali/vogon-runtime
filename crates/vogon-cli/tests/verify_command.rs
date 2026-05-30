use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn verify_command_accepts_support_triage_replay() {
    assert_verify_succeeds("support-triage");
}

#[test]
fn verify_command_accepts_writing_pipeline_replay() {
    assert_verify_succeeds("writing-pipeline");
}

#[test]
fn verify_command_rejects_mismatched_replay() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("mismatched-support-triage.replay.json");
    write_mismatched_replay("support-triage", &replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"mismatches\""));
    assert!(stderr.contains("\"step_output\""));
    assert!(stderr.contains("replay verification failed with"));
}

#[test]
fn verify_command_reports_missing_replay_path() {
    let missing_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("missing.replay.json");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&missing_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to read replay file"));
    assert!(stderr.contains(&missing_replay.display().to_string()));
}

#[test]
fn verify_command_reports_malformed_replay_path() {
    let malformed_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed.replay.json");
    write_malformed_replay(&malformed_replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&malformed_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains(&malformed_replay.display().to_string()));
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

fn write_mismatched_replay(name: &str, path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path(name)).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("drifted-output".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "{").unwrap();
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}
