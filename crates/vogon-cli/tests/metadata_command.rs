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
        "providers",
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
fn providers_help_documents_json_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("providers")
        .arg("--help")
        .output()
        .expect("providers help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in ["Show available model providers", "--json"] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn providers_command_reports_json_without_secret_values() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("providers")
        .arg("--json")
        .env("GEMINI_API_KEY", "secret-gemini-key")
        .env("GROQ_API_KEY", "secret-groq-key")
        .env("OPENAI_COMPATIBLE_API_KEY", "secret-openai-compatible-key")
        .output()
        .expect("providers command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("secret-gemini-key"));
    assert!(!stdout.contains("secret-groq-key"));
    assert!(!stdout.contains("secret-openai-compatible-key"));

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    let providers = report["providers"]
        .as_array()
        .expect("providers should be an array");
    assert!(providers.iter().any(|provider| {
        provider["name"] == "deterministic"
            && provider["enabled"] == true
            && provider["default"] == true
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "gemini"
            && provider["credential_env"] == "GEMINI_API_KEY"
            && provider["credential_configured"] == true
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "groq"
            && provider["credential_env"] == "GROQ_API_KEY"
            && provider["credential_configured"] == true
            && provider["default_base_url"] == "https://api.groq.com/openai/v1"
            && provider["default_model"] == "llama-3.1-8b-instant"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "openai-compatible"
            && provider["credential_env"] == "OPENAI_COMPATIBLE_API_KEY"
            && provider["credential_configured"] == true
    }));
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
        "--provider <PROVIDER>",
        "--gemini-model <GEMINI_MODEL>",
        "--gemini-timeout-seconds <GEMINI_TIMEOUT_SECONDS>",
        "--gemini-max-retries <GEMINI_MAX_RETRIES>",
        "--groq-model <GROQ_MODEL>",
        "--groq-timeout-seconds <GROQ_TIMEOUT_SECONDS>",
        "--groq-max-retries <GROQ_MAX_RETRIES>",
        "--openai-compatible-base-url <OPENAI_COMPATIBLE_BASE_URL>",
        "--openai-compatible-model <OPENAI_COMPATIBLE_MODEL>",
        "--openai-compatible-timeout-seconds <OPENAI_COMPATIBLE_TIMEOUT_SECONDS>",
        "--openai-compatible-max-retries <OPENAI_COMPATIBLE_MAX_RETRIES>",
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
fn verify_help_documents_json_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--help")
        .output()
        .expect("verify help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Verify a workflow against a replay file",
        "--json",
        "<WORKFLOW_FILE>",
        "<REPLAY_FILE>",
    ] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn check_help_documents_json_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg("--help")
        .output()
        .expect("check help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Validate a workflow file without executing it",
        "--json",
        "<WORKFLOW_FILE>",
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
