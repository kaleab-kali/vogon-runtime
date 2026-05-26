use std::{path::PathBuf, process::Command};

#[test]
fn run_command_executes_toml_workflow() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("workflows")
        .join("support-triage.toml");

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
