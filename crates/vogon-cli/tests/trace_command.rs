use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn support_triage_replay() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("replays")
        .join("support-triage.replay.json")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn classification_output() -> &'static str {
    "classify:25b99048d109fbed572129d473b8043dd72292d405951d7c0bb202a052a9a76d"
}

#[test]
fn trace_command_prints_replay_summary() {
    let replay = support_triage_replay();

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

#[test]
fn trace_command_can_emit_jsonl() {
    let replay = support_triage_replay();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg("--jsonl")
        .arg(replay)
        .output()
        .expect("trace command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["event"], "run");
    assert_eq!(lines[0]["workflow_name"], "support-triage");
    assert_eq!(lines[1]["event"], "step");
    assert_eq!(lines[1]["step_id"], "classify");
}

#[test]
fn trace_command_can_redact_human_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
        .arg(support_triage_replay())
        .output()
        .expect("trace command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[REDACTED:classification]"));
    assert!(!stdout.contains(classification_output()));
}

#[test]
fn trace_command_can_redact_jsonl_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg("--jsonl")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
        .arg(support_triage_replay())
        .output()
        .expect("trace command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(lines[1]["output"], "[REDACTED:classification]");
}

#[test]
fn trace_command_rejects_duplicate_redaction_labels() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg("--redact")
        .arg("classification=first")
        .arg("--redact")
        .arg("classification=second")
        .arg(support_triage_replay())
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("redaction label `classification` is configured more than once"));
}

#[test]
fn trace_command_reports_malformed_replay_path() {
    let malformed_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-trace.replay.json");
    write_malformed_replay(&malformed_replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&malformed_replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains(&malformed_replay.display().to_string()));
}

#[test]
fn trace_command_rejects_unknown_replay_fields() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unknown-trace-field.replay.json");
    write_unknown_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("unknown field `unexpected`"));
}

#[test]
fn trace_command_rejects_malformed_replay_hashes() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-trace-hash.replay.json");
    write_malformed_hash_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("must be 64 lowercase hexadecimal characters"));
}

#[test]
fn trace_command_rejects_malformed_replay_workflow_names() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-trace-workflow-name.replay.json");
    write_malformed_workflow_name_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("workflow name `support triage` contains unsupported characters"));
}

#[test]
fn trace_command_rejects_empty_replay_steps() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("empty-trace-steps.replay.json");
    write_empty_steps_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("replay must contain at least one step"));
}

#[test]
fn trace_command_rejects_duplicate_replay_step_ids() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("duplicate-trace-step-ids.replay.json");
    write_duplicate_step_ids_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("replay contains duplicate step id `classify`"));
}

#[test]
fn trace_command_rejects_malformed_redaction_markers() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-trace-redaction-marker.replay.json");
    write_malformed_redaction_marker_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malformed redaction marker"));
    assert!(stderr.contains("missing closing `]`"));
}

#[test]
fn trace_command_rejects_malformed_redaction_marker_labels() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-trace-redaction-marker-label.replay.json");
    write_malformed_redaction_marker_label_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("trace")
        .arg(&replay)
        .output()
        .expect("trace command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malformed redaction marker"));
    assert!(stderr.contains("invalid redaction label `bad label`"));
}

fn write_malformed_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "{").unwrap();
}

fn write_unknown_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["unexpected"] = serde_json::Value::String("ignored-before-strict-parsing".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_hash_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["steps"][0]["input_hash"] = serde_json::Value::String("not-a-sha256-hash".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_workflow_name_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["workflow_name"] = serde_json::Value::String("support triage".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_empty_steps_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["steps"] = serde_json::Value::Array(Vec::new());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_duplicate_step_ids_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["steps"][1]["step_id"] = replay["steps"][0]["step_id"].clone();
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_redaction_marker_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("[REDACTED:classification".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_redaction_marker_label_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(support_triage_replay()).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("[REDACTED:bad label]".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}
