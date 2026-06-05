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
fn verify_command_accepts_redacted_replay() {
    let redacted_replay =
        write_redacted_support_triage_replay("redacted-support-triage.replay.json");

    let replay_text = fs::read_to_string(&redacted_replay).unwrap();
    assert!(replay_text.contains("[REDACTED:classification]"));
    assert!(!replay_text.contains(classification_output()));

    let verify_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
        .arg(workflow_path("support-triage"))
        .arg(&redacted_replay)
        .output()
        .expect("verify command should execute");

    assert!(
        verify_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    assert!(String::from_utf8_lossy(&verify_output.stdout).contains("Replay verified"));
}

#[test]
fn verify_command_rejects_whitespace_redaction_labels() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact")
        .arg(format!(" classification ={}", classification_output()))
        .arg(workflow_path("support-triage"))
        .arg(replay_path("support-triage"))
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("redaction label ` classification ` contains unsupported characters"));
}

#[test]
fn verify_command_rejects_redacted_replay_without_matching_redaction() {
    let redacted_replay =
        write_redacted_support_triage_replay("unconfigured-redacted-support-triage.replay.json");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&redacted_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("without matching --redact label(s): classification"));
    assert!(!stderr.contains("\"mismatches\""));
    assert!(!stderr.contains(classification_output()));
}

#[test]
fn verify_command_rejects_malformed_redaction_marker_before_execution() {
    let malformed_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-redaction-marker.replay.json");
    write_malformed_redaction_marker_replay(&malformed_replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&malformed_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malformed redaction marker"));
    assert!(stderr.contains("missing closing `]`"));
    assert!(!stderr.contains("\"mismatches\""));
    assert!(!stderr.contains(classification_output()));
}

#[test]
fn verify_command_masks_redacted_replay_mismatch_outputs() {
    let redacted_replay =
        write_redacted_support_triage_replay("wrong-redaction-redacted-support-triage.replay.json");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact")
        .arg("classification=wrong-output")
        .arg(workflow_path("support-triage"))
        .arg(&redacted_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"step_output\""));
    assert!(stderr.contains("[UNREPORTED: replay is redacted]"));
    assert!(!stderr.contains(classification_output()));
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

#[test]
fn verify_command_rejects_unknown_top_level_replay_fields() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unknown-top-level-field.replay.json");
    write_unknown_top_level_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("unknown field `unexpected`"));
}

#[test]
fn verify_command_rejects_unknown_step_replay_fields() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unknown-step-field.replay.json");
    write_unknown_step_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("unknown field `unexpected`"));
}

#[test]
fn verify_command_rejects_malformed_replay_hashes() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-hash.replay.json");
    write_malformed_hash_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("must be 64 lowercase hexadecimal characters"));
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

fn classification_output() -> &'static str {
    "classify:25b99048d109fbed572129d473b8043dd72292d405951d7c0bb202a052a9a76d"
}

fn write_redacted_support_triage_replay(file_name: &str) -> PathBuf {
    let redacted_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join(file_name);
    fs::create_dir_all(redacted_replay.parent().unwrap()).unwrap();

    let run_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
        .arg("--output")
        .arg(&redacted_replay)
        .arg(workflow_path("support-triage"))
        .output()
        .expect("run command should execute");

    assert!(
        run_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    redacted_replay
}

fn write_mismatched_replay(name: &str, path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path(name)).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("drifted-output".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_redaction_marker_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("[REDACTED:classification".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "{").unwrap();
}

fn write_unknown_top_level_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["unexpected"] = serde_json::Value::String("ignored-before-strict-parsing".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_unknown_step_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][0]["unexpected"] =
        serde_json::Value::String("ignored-before-strict-parsing".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_hash_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["run_hash"] = serde_json::Value::String("not-a-sha256-hash".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}
