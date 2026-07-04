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
        "doctor",
        "init",
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
fn init_help_documents_output_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("init")
        .arg("--help")
        .output()
        .expect("init help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Create a starter TOML workflow file",
        "--output <FILE>",
        "--force",
    ] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn doctor_help_documents_json_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("doctor")
        .arg("--help")
        .output()
        .expect("doctor help should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in ["Run local installation diagnostics", "--json"] {
        assert!(
            stdout.contains(expected),
            "stdout should contain `{expected}`:\n{stdout}"
        );
    }
}

#[test]
fn doctor_command_reports_json_without_secret_values() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("doctor")
        .arg("--json")
        .env("GEMINI_API_KEY", "secret-gemini-key")
        .env("GROQ_API_KEY", "secret-groq-key")
        .env("HF_TOKEN", "secret-hugging-face-token")
        .env("OPENROUTER_API_KEY", "secret-openrouter-key")
        .env("OPENAI_COMPATIBLE_API_KEY", "secret-openai-compatible-key")
        .output()
        .expect("doctor command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("secret-gemini-key"));
    assert!(!stdout.contains("secret-groq-key"));
    assert!(!stdout.contains("secret-hugging-face-token"));
    assert!(!stdout.contains("secret-openrouter-key"));
    assert!(!stdout.contains("secret-openai-compatible-key"));

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["status"], "ok");
    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| { check["name"] == "deterministic_runtime" && check["status"] == "ok" })
    );
    assert!(
        report["providers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|provider| {
                provider["name"] == "deterministic"
                    && provider["enabled"] == true
                    && provider["default"] == true
                    && provider["documentation_url"]
                        == "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic"
                    && provider["usage_url"].is_null()
            })
    );
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
        .env("HF_TOKEN", "secret-hugging-face-token")
        .env("OPENROUTER_API_KEY", "secret-openrouter-key")
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
    assert!(!stdout.contains("secret-hugging-face-token"));
    assert!(!stdout.contains("secret-openrouter-key"));
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
            && provider["documentation_url"]
                == "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "gemini"
            && provider["credential_env"] == "GEMINI_API_KEY"
            && provider["credential_configured"] == true
            && provider["documentation_url"] == "https://ai.google.dev/gemini-api/docs"
            && provider["usage_url"] == "https://ai.google.dev/gemini-api/docs/pricing"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "groq"
            && provider["credential_env"] == "GROQ_API_KEY"
            && provider["credential_configured"] == true
            && provider["default_base_url"] == "https://api.groq.com/openai/v1"
            && provider["default_model"] == "llama-3.1-8b-instant"
            && provider["documentation_url"] == "https://console.groq.com/docs/openai"
            && provider["usage_url"] == "https://console.groq.com/docs/rate-limits"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "hugging-face"
            && provider["credential_env"] == "HF_TOKEN"
            && provider["credential_configured"] == true
            && provider["default_base_url"] == "https://router.huggingface.co/v1"
            && provider["default_model"] == "openai/gpt-oss-120b:fastest"
            && provider["documentation_url"] == "https://huggingface.co/docs/inference-providers"
            && provider["usage_url"] == "https://huggingface.co/docs/inference-providers/pricing"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "openrouter"
            && provider["credential_env"] == "OPENROUTER_API_KEY"
            && provider["credential_configured"] == true
            && provider["default_base_url"] == "https://openrouter.ai/api/v1"
            && provider["default_model"] == "openrouter/free"
            && provider["documentation_url"] == "https://openrouter.ai/docs"
            && provider["usage_url"] == "https://openrouter.ai/pricing"
    }));
    assert!(providers.iter().any(|provider| {
        provider["name"] == "openai-compatible"
            && provider["credential_env"] == "OPENAI_COMPATIBLE_API_KEY"
            && provider["credential_configured"] == true
            && provider["documentation_url"]
                == "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible"
            && provider["usage_url"].is_null()
    }));
}

#[test]
fn providers_command_prints_documentation_links() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("providers")
        .output()
        .expect("providers command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "documentation: https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic",
        "documentation: https://ai.google.dev/gemini-api/docs",
        "documentation: https://console.groq.com/docs/openai",
        "documentation: https://huggingface.co/docs/inference-providers",
        "documentation: https://openrouter.ai/docs",
        "documentation: https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible",
        "usage and limits: https://ai.google.dev/gemini-api/docs/pricing",
        "usage and limits: https://console.groq.com/docs/rate-limits",
        "usage and limits: https://huggingface.co/docs/inference-providers/pricing",
        "usage and limits: https://openrouter.ai/pricing",
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
        "--provider <PROVIDER>",
        "--gemini-model <GEMINI_MODEL>",
        "--gemini-timeout-seconds <GEMINI_TIMEOUT_SECONDS>",
        "--gemini-max-retries <GEMINI_MAX_RETRIES>",
        "--groq-model <GROQ_MODEL>",
        "--groq-timeout-seconds <GROQ_TIMEOUT_SECONDS>",
        "--groq-max-retries <GROQ_MAX_RETRIES>",
        "--hugging-face-model <HUGGING_FACE_MODEL>",
        "--hugging-face-timeout-seconds <HUGGING_FACE_TIMEOUT_SECONDS>",
        "--hugging-face-max-retries <HUGGING_FACE_MAX_RETRIES>",
        "--openrouter-model <OPENROUTER_MODEL>",
        "--openrouter-timeout-seconds <OPENROUTER_TIMEOUT_SECONDS>",
        "--openrouter-max-retries <OPENROUTER_MAX_RETRIES>",
        "--openai-compatible-base-url <OPENAI_COMPATIBLE_BASE_URL>",
        "--openai-compatible-model <OPENAI_COMPATIBLE_MODEL>",
        "--openai-compatible-timeout-seconds <OPENAI_COMPATIBLE_TIMEOUT_SECONDS>",
        "--openai-compatible-max-retries <OPENAI_COMPATIBLE_MAX_RETRIES>",
        "--redact <LABEL=VALUE>",
        "--output <FILE>",
        "--cache-file <FILE>",
        "--cache-max-entries <CACHE_MAX_ENTRIES>",
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
