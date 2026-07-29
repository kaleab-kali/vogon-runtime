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
fn verify_command_renders_the_same_workflow_inputs_as_run() {
    let workflow = repo_root()
        .join("fixtures")
        .join("workflows")
        .join("git-change-review.toml");
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("input-rendered.replay.json");
    fs::create_dir_all(replay.parent().unwrap()).unwrap();

    let run_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--input")
        .arg("git_diff=timeout_seconds changed from 30 to 0")
        .arg("--output")
        .arg(&replay)
        .arg(&workflow)
        .output()
        .expect("run command should execute");
    assert!(
        run_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let verify_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--input")
        .arg("git_diff=timeout_seconds changed from 30 to 0")
        .arg(&workflow)
        .arg(&replay)
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
fn verify_command_rejects_replay_file_as_cache_file() {
    let replay = replay_path("support-triage");
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--cache-file")
        .arg(&replay)
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("replay file path"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("run cache path"));
}

#[test]
fn verify_command_can_emit_json_match_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--json")
        .arg(workflow_path("support-triage"))
        .arg(replay_path("support-triage"))
        .output()
        .expect("verify command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["is_match"], true);
    assert_eq!(report["mismatches"].as_array().unwrap().len(), 0);
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
fn verify_command_accepts_environment_redaction_for_redacted_replay() {
    let redacted_replay =
        write_redacted_support_triage_replay("environment-redacted-support-triage.replay.json");

    let verify_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact-env")
        .arg("classification=VOGON_TEST_REDACTION")
        .arg(workflow_path("support-triage"))
        .arg(&redacted_replay)
        .env("VOGON_TEST_REDACTION", classification_output())
        .output()
        .expect("verify command should execute");

    assert!(
        verify_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    assert!(String::from_utf8_lossy(&verify_output.stdout).contains("Replay verified"));
    assert!(!String::from_utf8_lossy(&verify_output.stderr).contains(classification_output()));
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
fn verify_command_rejects_duplicate_redaction_labels() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact")
        .arg("classification=first")
        .arg("--redact")
        .arg("classification=second")
        .arg(workflow_path("support-triage"))
        .arg(replay_path("support-triage"))
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate redaction label `classification`"));
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
    assert!(stderr.contains("without matching --redact or --redact-env label(s): classification"));
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
fn verify_command_rejects_malformed_redaction_marker_labels() {
    let malformed_replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-redaction-marker-label.replay.json");
    write_malformed_redaction_marker_label_replay(&malformed_replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&malformed_replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("malformed redaction marker"));
    assert!(stderr.contains("invalid redaction label `bad label`"));
    assert!(!stderr.contains("\"mismatches\""));
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
fn verify_command_redacts_expected_mismatch_outputs() {
    let secret = "sk-expected-secret";
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("sensitive-expected-mismatch.replay.json");
    write_sensitive_expected_mismatch_replay(&replay, secret);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--redact")
        .arg(format!("secret={secret}"))
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[REDACTED:secret]"));
    assert!(!stderr.contains(secret));
}

#[test]
fn verify_command_redacts_expected_json_mismatch_outputs() {
    let secret = "sk-json-expected-secret";
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("json-sensitive-expected-mismatch.replay.json");
    write_sensitive_expected_mismatch_replay(&replay, secret);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--json")
        .arg("--redact")
        .arg(format!("secret={secret}"))
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[REDACTED:secret]"));
    assert!(!stdout.contains(secret));
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
fn verify_command_reports_runtime_metadata_provider_mismatches() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("provider-mismatch-support-triage.replay.json");
    write_gemini_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--provider")
        .arg("deterministic")
        .arg(workflow_path("support-triage"))
        .arg(replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("\"runtime_metadata\""));
    assert!(stderr.contains("\"expected\""));
    assert!(stderr.contains("\"gemini\""));
    assert!(stderr.contains("\"actual\""));
    assert!(stderr.contains("\"deterministic\""));
}

#[test]
fn verify_command_defaults_to_replay_provider_metadata() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("auto-gemini-provider-support-triage.replay.json");
    write_gemini_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(replay)
        .env_remove("GEMINI_API_KEY")
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("GEMINI_API_KEY must be set"));
}

#[test]
fn verify_command_defaults_to_groq_replay_provider_metadata() {
    let replay = std::env::temp_dir()
        .join("vogon-cli-tests")
        .join("auto-groq-provider-support-triage.replay.json");
    write_groq_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .env_remove("GROQ_API_KEY")
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("GROQ_API_KEY must be set"));
}

#[test]
fn verify_command_defaults_to_hugging_face_replay_provider_metadata() {
    let replay = std::env::temp_dir()
        .join("vogon-cli-tests")
        .join("auto-hugging-face-provider-support-triage.replay.json");
    write_hugging_face_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .env_remove("HF_TOKEN")
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("HF_TOKEN must be set"));
}

#[test]
fn verify_command_defaults_to_nvidia_replay_provider_metadata() {
    let replay = std::env::temp_dir()
        .join("vogon-cli-tests")
        .join("auto-nvidia-provider-support-triage.replay.json");
    write_nvidia_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .env_remove("NVIDIA_API_KEY")
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NVIDIA_API_KEY must be set"));
}

#[test]
fn verify_command_defaults_to_openrouter_replay_provider_metadata() {
    let replay = std::env::temp_dir()
        .join("vogon-cli-tests")
        .join("auto-openrouter-provider-support-triage.replay.json");
    write_openrouter_metadata_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OPENROUTER_API_KEY must be set"));
}

#[test]
fn verify_command_can_emit_json_mismatch_report() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("json-mismatched-support-triage.replay.json");
    write_mismatched_replay("support-triage", &replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--json")
        .arg(workflow_path("support-triage"))
        .arg(replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["is_match"], false);
    assert!(!report["mismatches"].as_array().unwrap().is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
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
fn verify_command_rejects_unsupported_replay_schema_versions() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unsupported-schema-version.replay.json");
    write_unsupported_schema_version_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("unsupported replay schema_version `99`"));
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

#[test]
fn verify_command_rejects_malformed_replay_workflow_names() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed-workflow-name.replay.json");
    write_malformed_workflow_name_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("workflow name `support triage` contains unsupported characters"));
}

#[test]
fn verify_command_rejects_empty_replay_steps() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("empty-steps.replay.json");
    write_empty_steps_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("replay must contain at least one step"));
}

#[test]
fn verify_command_rejects_duplicate_replay_step_ids() {
    let replay = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("duplicate-step-ids.replay.json");
    write_duplicate_step_ids_replay(&replay);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(workflow_path("support-triage"))
        .arg(&replay)
        .output()
        .expect("verify command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse replay file"));
    assert!(stderr.contains("replay contains duplicate step id `classify`"));
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

fn write_gemini_metadata_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["runtime"] = serde_json::json!({
        "provider": "gemini",
        "adapter": "gemini-generate-content",
        "adapter_version": "0.1.0",
        "model": "gemini-3.1-flash-lite",
        "cache_identity": "vogon-adapters@0.1.0:gemini:v1:base=https://generativelanguage.googleapis.com:model=gemini-3.1-flash-lite:timeout_nanos=30000000000:max_retries=2",
        "parameters": {
            "base_url": "https://generativelanguage.googleapis.com",
            "timeout_nanos": "30000000000",
            "max_retries": "2"
        }
    });
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_groq_metadata_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["runtime"] = serde_json::json!({
        "provider": "groq",
        "adapter": "groq-openai-compatible-chat-completions",
        "adapter_version": "0.1.0",
        "model": "llama-3.1-8b-instant",
        "cache_identity": "vogon-adapters@0.1.0:groq:v1:base=https://api.groq.com/openai/v1:model=llama-3.1-8b-instant:timeout_nanos=30000000000:max_retries=2",
        "parameters": {
            "base_url": "https://api.groq.com/openai/v1",
            "timeout_nanos": "30000000000",
            "max_retries": "2"
        }
    });
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_hugging_face_metadata_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["runtime"] = serde_json::json!({
        "provider": "hugging-face",
        "adapter": "hugging-face-openai-compatible-chat-completions",
        "adapter_version": "0.1.0",
        "model": "openai/gpt-oss-120b:fastest",
        "cache_identity": "vogon-adapters@0.1.0:hugging-face:v1:base=https://router.huggingface.co/v1:model=openai/gpt-oss-120b:fastest:timeout_nanos=30000000000:max_retries=2",
        "parameters": {
            "base_url": "https://router.huggingface.co/v1",
            "timeout_nanos": "30000000000",
            "max_retries": "2"
        }
    });
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_nvidia_metadata_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["runtime"] = serde_json::json!({
        "provider": "nvidia",
        "adapter": "nvidia-openai-compatible-chat-completions",
        "adapter_version": "0.1.0",
        "model": "meta/llama-3.1-8b-instruct",
        "cache_identity": "vogon-adapters@0.1.0:nvidia:v1:base=https://integrate.api.nvidia.com/v1:model=meta/llama-3.1-8b-instruct:timeout_nanos=30000000000:max_retries=2",
        "parameters": {
            "base_url": "https://integrate.api.nvidia.com/v1",
            "timeout_nanos": "30000000000",
            "max_retries": "2"
        }
    });
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_openrouter_metadata_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["runtime"] = serde_json::json!({
        "provider": "openrouter",
        "adapter": "openrouter-openai-compatible-chat-completions",
        "adapter_version": "0.1.0",
        "model": "openrouter/free",
        "cache_identity": "vogon-adapters@0.1.0:openrouter:v1:base=https://openrouter.ai/api/v1:model=openrouter/free:timeout_nanos=30000000000:max_retries=2",
        "parameters": {
            "base_url": "https://openrouter.ai/api/v1",
            "timeout_nanos": "30000000000",
            "max_retries": "2"
        }
    });
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_sensitive_expected_mismatch_replay(path: &Path, secret: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String(format!("expected token {secret}"));
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_redaction_marker_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("[REDACTED:classification".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_malformed_redaction_marker_label_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][0]["output"] = serde_json::Value::String("[REDACTED:bad label]".to_owned());
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

fn write_unsupported_schema_version_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["schema_version"] = serde_json::Value::Number(99.into());
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

fn write_malformed_workflow_name_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["workflow_name"] = serde_json::Value::String("support triage".to_owned());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_empty_steps_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"] = serde_json::Value::Array(Vec::new());
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn write_duplicate_step_ids_replay(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_path("support-triage")).unwrap()).unwrap();
    replay["steps"][1]["step_id"] = replay["steps"][0]["step_id"].clone();
    fs::write(path, serde_json::to_string_pretty(&replay).unwrap()).unwrap();
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}
