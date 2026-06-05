use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn support_triage_workflow() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join("support-triage.toml")
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

fn write_malformed_workflow(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "name = [").unwrap();
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
