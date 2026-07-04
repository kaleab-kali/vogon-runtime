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

fn classification_output() -> &'static str {
    "classify:25b99048d109fbed572129d473b8043dd72292d405951d7c0bb202a052a9a76d"
}

fn remove_file_if_exists(path: &Path) {
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

fn remove_path_if_exists(path: &Path) {
    if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
    } else if path.exists() {
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

    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["runtime"]["provider"], "deterministic");
    assert_eq!(report["runtime"]["model"], "deterministic-echo");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_redacts_known_output_literals() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
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
    assert!(!stdout.contains(classification_output()));
}

#[test]
fn run_command_prefers_longest_overlapping_redaction_literals() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg("prefix=classify:25b99048")
        .arg("--redact")
        .arg(format!("classification={}", classification_output()))
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
    assert!(!stdout.contains("[REDACTED:prefix]d109fbed"));
    assert!(!stdout.contains(classification_output()));
}

#[test]
fn run_command_rejects_duplicate_redaction_labels() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg("classification=first")
        .arg("--redact")
        .arg("classification=second")
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate redaction label `classification`"));
}

#[test]
fn run_command_reports_missing_gemini_api_key() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("gemini")
        .arg(fixture)
        .env_remove("GEMINI_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("GEMINI_API_KEY must be set"));
}

#[test]
fn run_command_rejects_zero_gemini_timeout() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("gemini")
        .arg("--gemini-timeout-seconds")
        .arg("0")
        .arg(fixture)
        .env_remove("GEMINI_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"));
}

#[test]
fn run_command_rejects_excessive_gemini_retry_count() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("gemini")
        .arg("--gemini-max-retries")
        .arg("21")
        .arg(fixture)
        .env_remove("GEMINI_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"));
    assert!(stderr.contains("--gemini-max-retries must be between 0 and 20"));
}

#[test]
fn run_command_reports_missing_openai_compatible_api_key() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg(fixture)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OPENAI_COMPATIBLE_API_KEY must be set"));
}

#[test]
fn run_command_reports_missing_groq_api_key() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("groq")
        .arg(support_triage_workflow())
        .env_remove("GROQ_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("GROQ_API_KEY must be set"));
}

#[test]
fn run_command_reports_missing_hugging_face_token() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("hugging-face")
        .arg(support_triage_workflow())
        .env_remove("HF_TOKEN")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("HF_TOKEN must be set"));
}

#[test]
fn run_command_reports_missing_openrouter_api_key() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openrouter")
        .arg(support_triage_workflow())
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OPENROUTER_API_KEY must be set"));
}

#[test]
fn run_command_rejects_zero_groq_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("groq")
        .arg("--groq-timeout-seconds")
        .arg("0")
        .arg(support_triage_workflow())
        .env_remove("GROQ_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value '0'"));
}

#[test]
fn run_command_rejects_zero_hugging_face_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("hugging-face")
        .arg("--hugging-face-timeout-seconds")
        .arg("0")
        .arg(support_triage_workflow())
        .env_remove("HF_TOKEN")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value '0'"));
}

#[test]
fn run_command_rejects_zero_openrouter_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openrouter")
        .arg("--openrouter-timeout-seconds")
        .arg("0")
        .arg(support_triage_workflow())
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value '0'"));
}

#[test]
fn run_command_rejects_excessive_groq_retry_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("groq")
        .arg("--groq-max-retries")
        .arg("21")
        .arg(support_triage_workflow())
        .env_remove("GROQ_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--groq-max-retries must be between 0 and 20"));
}

#[test]
fn run_command_rejects_excessive_hugging_face_retry_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("hugging-face")
        .arg("--hugging-face-max-retries")
        .arg("21")
        .arg(support_triage_workflow())
        .env_remove("HF_TOKEN")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--hugging-face-max-retries must be between 0 and 20"));
}

#[test]
fn run_command_rejects_excessive_openrouter_retry_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openrouter")
        .arg("--openrouter-max-retries")
        .arg("21")
        .arg(support_triage_workflow())
        .env_remove("OPENROUTER_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--openrouter-max-retries must be between 0 and 20"));
}

#[test]
fn run_command_rejects_zero_openai_compatible_timeout() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-timeout-seconds")
        .arg("0")
        .arg(fixture)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"));
}

#[test]
fn run_command_rejects_excessive_openai_compatible_retry_count() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-max-retries")
        .arg("21")
        .arg(fixture)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"));
    assert!(stderr.contains("--openai-compatible-max-retries must be between 0 and 20"));
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
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["workflow_name"], "support-triage");
    assert_eq!(report["runtime"]["provider"], "deterministic");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_persists_run_cache_file() {
    let fixture = support_triage_workflow();
    let cache_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("support-triage.cache.json");
    remove_file_if_exists(&cache_file);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cache: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cache_file).unwrap()).unwrap();
    assert_eq!(cache["max_entries"], 1024);
    assert_eq!(cache["outputs"].as_object().unwrap().len(), 2);
    assert_eq!(cache["insertion_order"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_applies_cache_entry_limit_when_loading_cache_file() {
    let fixture = support_triage_workflow();
    let cache_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("bounded-support-triage.cache.json");
    remove_file_if_exists(&cache_file);

    let first = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg(&fixture)
        .output()
        .expect("run command should execute");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg("--cache-max-entries")
        .arg("1")
        .arg(fixture)
        .output()
        .expect("run command should execute");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let cache: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cache_file).unwrap()).unwrap();
    assert_eq!(cache["max_entries"], 1);
    assert_eq!(cache["outputs"].as_object().unwrap().len(), 1);
    assert_eq!(cache["insertion_order"].as_array().unwrap().len(), 1);
}

#[test]
fn run_command_rejects_malformed_cache_file() {
    let fixture = support_triage_workflow();
    let cache_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("malformed.cache.json");
    fs::create_dir_all(cache_file.parent().unwrap()).unwrap();
    fs::write(&cache_file, "not json").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to parse run cache file"));
    assert!(stderr.contains(&cache_file.display().to_string()));
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

#[test]
fn run_command_reports_output_parent_errors() {
    let fixture = support_triage_workflow();
    let blocked_parent = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("not-a-directory");
    remove_path_if_exists(&blocked_parent);
    fs::create_dir_all(blocked_parent.parent().unwrap()).unwrap();
    fs::write(&blocked_parent, "blocks output directory creation").unwrap();

    let output_file = blocked_parent.join("support-triage.replay.json");
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--output")
        .arg(&output_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed to create replay output directory"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(&blocked_parent.display().to_string()),
        "stderr: {stderr}"
    );
}

#[test]
fn run_command_rejects_directory_output_path() {
    let fixture = support_triage_workflow();
    let output_dir = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("directory-output.replay.json");
    remove_path_if_exists(&output_dir);
    fs::create_dir_all(&output_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--output")
        .arg(&output_dir)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("replay output path"));
    assert!(stderr.contains("is a directory"));
    assert!(
        stderr.contains(&output_dir.display().to_string()),
        "stderr: {stderr}"
    );
}
