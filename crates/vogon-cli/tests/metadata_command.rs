use std::process::Command;

#[test]
fn help_flag_lists_public_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("--help")
        .output()
        .expect("help flag should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Deterministic, replayable AI workflow runtime.",
        "Commands:",
        "check",
        "demo",
        "run",
        "verify",
        "trace",
        "--version",
    ] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn run_help_documents_replay_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--help")
        .output()
        .expect("run help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Run a workflow file",
        "<WORKFLOW_FILE>",
        "--redact <LABEL=VALUE>",
        "--output <FILE>",
    ] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn version_flag_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("--version")
        .output()
        .expect("version flag should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("vogon {}", env!("CARGO_PKG_VERSION"))
    );
}
