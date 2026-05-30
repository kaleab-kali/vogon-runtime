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

fn remove_file_if_exists(path: &Path) {
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn run_command_executes_toml_workflow() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");

    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_redacts_known_output_literals() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg("classification=classify:25b99048d109fbed572129d473b8043dd72292d405951d7c0bb202a052a9a76d")
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[REDACTED:classification]"));
    assert!(!stdout.contains(
        "output\": \"classify:25b99048d109fbed572129d473b8043dd72292d405951d7c0bb202a052a9a76d"
    ));
}

#[test]
fn run_command_writes_replay_file() {
    let fixture = support_triage_workflow();
    let output_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("support-triage.replay.json");
    remove_file_if_exists(&output_file);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--output")
        .arg(&output_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Replay written:"));

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output_file).unwrap()).unwrap();
    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_overwrites_existing_replay_file() {
    let fixture = support_triage_workflow();
    let output_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("existing-support-triage.replay.json");
    fs::create_dir_all(output_file.parent().unwrap()).unwrap();
    fs::write(&output_file, "old replay").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--output")
        .arg(&output_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output_file).unwrap()).unwrap();
    assert_eq!(report["workflow_name"], "support-triage");
}
