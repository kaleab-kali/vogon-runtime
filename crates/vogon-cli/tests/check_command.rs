use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const OVERSIZED_INPUT_BYTES: usize = 1024 * 1024 + 1;

fn support_triage_workflow() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join("support-triage.toml")
}

fn release_gate_workflow() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join("release-gate.toml")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_invalid_workflow(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "invalid"

[[steps]]
id = "duplicate"
prompt = "First"

[[steps]]
id = "duplicate"
prompt = "Second"
"#,
    )
    .unwrap();
}

fn write_workflow_with_empty_prompt(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "invalid"

[[steps]]
id = "empty_prompt"
prompt = " "
"#,
    )
    .unwrap();
}

fn write_workflow_with_whitespace_step_id(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "invalid"

[[steps]]
id = " classify "
prompt = "Classify"
"#,
    )
    .unwrap();
}

fn write_workflow_with_whitespace_name(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = " support "

[[steps]]
id = "classify"
prompt = "Classify"
"#,
    )
    .unwrap();
}

fn write_workflow_with_invalid_name_characters(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "support triage"

[[steps]]
id = "classify"
prompt = "Classify"
"#,
    )
    .unwrap();
}

fn write_malformed_workflow(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "name = [").unwrap();
}

fn write_oversized_file(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "x".repeat(OVERSIZED_INPUT_BYTES)).unwrap();
}

fn write_workflow_with_unknown_top_level_field(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "invalid"
unexpected = true

[[steps]]
id = "classify"
prompt = "Classify"
"#,
    )
    .unwrap();
}

fn write_workflow_with_unknown_step_field(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"
name = "invalid"

[[steps]]
id = "classify"
prompt = "Classify"
temperature = 0.7
"#,
    )
    .unwrap();
}

#[test]
fn check_command_accepts_valid_toml_workflow() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(support_triage_workflow())
        .output()
        .expect("check command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Workflow valid: support-triage (2 steps)")
    );
}

#[test]
fn check_command_can_emit_json_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg("--json")
        .arg(support_triage_workflow())
        .output()
        .expect("check command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(summary["workflow_name"], "support-triage");
    assert_eq!(summary["step_count"], 2);
    assert!(summary.get("required_inputs").is_none());
}

#[test]
fn check_command_reports_required_workflow_inputs() {
    let workflow = repo_root()
        .join("fixtures")
        .join("workflows")
        .join("git-change-review.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg("--json")
        .arg(workflow)
        .output()
        .expect("check command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(summary["workflow_name"], "git-change-review");
    assert_eq!(summary["step_count"], 2);
    assert_eq!(summary["required_inputs"], serde_json::json!(["git_diff"]));
}

#[test]
fn check_command_reports_the_decision_policy() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg("--json")
        .arg(release_gate_workflow())
        .output()
        .expect("check command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(summary["decision"]["step"], "release_decision");
    assert_eq!(summary["decision"]["pointer"], "/decision");
    assert_eq!(summary["decision"]["allow"], serde_json::json!(["GO"]));
    assert_eq!(summary["decision"]["deny"], serde_json::json!(["NO_GO"]));
}

#[test]
fn check_command_rejects_invalid_toml_workflow() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("invalid-workflow.toml");
    write_invalid_workflow(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate step id `duplicate`"));
}

#[test]
fn check_command_rejects_empty_step_prompt() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("empty-prompt-workflow.toml");
    write_workflow_with_empty_prompt(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("step `empty_prompt` prompt cannot be empty")
    );
}

#[test]
fn check_command_rejects_whitespace_padded_step_ids() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("whitespace-step-id-workflow.toml");
    write_workflow_with_whitespace_step_id(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported characters"));
}

#[test]
fn check_command_rejects_whitespace_padded_workflow_names() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("whitespace-workflow-name.toml");
    write_workflow_with_whitespace_name(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("workflow name ` support ` must not have leading or trailing whitespace")
    );
}

#[test]
fn check_command_rejects_workflow_names_with_unsupported_characters() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unsupported-workflow-name.toml");
    write_workflow_with_invalid_name_characters(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("workflow name `support triage` contains unsupported characters")
    );
}

#[test]
fn check_command_reports_missing_workflow_path() {
    let missing_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("missing-workflow.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(&missing_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to read workflow file"));
    assert!(stderr.contains(&missing_workflow.display().to_string()));
}

#[test]
fn check_command_rejects_oversized_workflow_file() {
    let oversized_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("oversized-workflow.toml");
    write_oversized_file(&oversized_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(&oversized_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("workflow file"));
    assert!(stderr.contains("exceeding the 1 MiB limit"));
    assert!(stderr.contains(&oversized_workflow.display().to_string()));
}

#[test]
fn check_command_reports_malformed_workflow_path() {
    let malformed_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-workflow.toml");
    write_malformed_workflow(&malformed_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(&malformed_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse workflow file"));
    assert!(stderr.contains(&malformed_workflow.display().to_string()));
}

#[test]
fn check_command_rejects_unknown_top_level_fields() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unknown-top-level-field-workflow.toml");
    write_workflow_with_unknown_top_level_field(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(&invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse workflow file"));
    assert!(stderr.contains("unexpected"));
}

#[test]
fn check_command_rejects_unknown_step_fields() {
    let invalid_workflow = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unknown-step-field-workflow.toml");
    write_workflow_with_unknown_step_field(&invalid_workflow);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg(&invalid_workflow)
        .output()
        .expect("check command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse workflow file"));
    assert!(stderr.contains("temperature"));
}
