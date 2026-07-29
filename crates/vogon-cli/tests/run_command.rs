use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    thread,
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

fn git_change_review_workflow() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join("git-change-review.toml")
}

fn release_gate_workflow() -> PathBuf {
    repo_root()
        .join("fixtures")
        .join("workflows")
        .join("release-gate.toml")
}

fn spawn_openai_compatible_server(outputs: Vec<&'static str>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        for output in outputs {
            let (mut stream, _) = listener.accept().unwrap();
            read_http_request(&mut stream);
            let body = serde_json::json!({
                "choices": [{"message": {"content": output}}]
            })
            .to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    (format!("http://{address}"), server)
}

fn create_git_diff_repository(name: &str) -> PathBuf {
    let repository = repo_root().join("target").join("vogon-tests").join(name);
    remove_path_if_exists(&repository);
    fs::create_dir_all(&repository).unwrap();
    fs::write(repository.join("service.toml"), "timeout_seconds = 30\n").unwrap();

    for arguments in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Vogon Tests"],
        vec!["config", "user.email", "vogon-tests@example.invalid"],
        vec!["add", "service.toml"],
        vec!["commit", "-m", "Add baseline"],
    ] {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(&repository)
            .output()
            .expect("Git should execute");
        assert!(
            output.status.success(),
            "Git stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    repository
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
fn run_command_enforces_an_allowed_structured_decision() {
    let (base_url, server) =
        spawn_openai_compatible_server(vec!["no blocking risks", r#"{"decision":"GO"}"#]);
    let replay_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("allowed-release-gate.replay.json");
    remove_file_if_exists(&replay_file);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-base-url")
        .arg(base_url)
        .arg("--openai-compatible-model")
        .arg("local/model")
        .arg("--openai-compatible-no-auth")
        .arg("--openai-compatible-max-retries")
        .arg("0")
        .arg("--input")
        .arg("git_diff=timeout_seconds changed from 30 to 45")
        .arg("--enforce-decision")
        .arg("--output")
        .arg(&replay_file)
        .arg(release_gate_workflow())
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");
    server.join().unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_file).unwrap()).unwrap();
    assert_eq!(replay["decision"]["value"], "GO");
    assert_eq!(replay["decision"]["outcome"], "allow");
    assert_eq!(replay["decision"]["step_id"], "release_decision");
}

#[test]
fn run_command_writes_denied_decision_before_failing_the_gate() {
    let (base_url, server) = spawn_openai_compatible_server(vec![
        "rollback plan is missing",
        r#"{"decision":"NO_GO","required_actions":["add rollback plan"]}"#,
    ]);
    let replay_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("denied-release-gate.replay.json");
    remove_file_if_exists(&replay_file);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-base-url")
        .arg(base_url)
        .arg("--openai-compatible-model")
        .arg("local/model")
        .arg("--openai-compatible-no-auth")
        .arg("--openai-compatible-max-retries")
        .arg("0")
        .arg("--input")
        .arg("git_diff=removed rollback handling")
        .arg("--enforce-decision")
        .arg("--output")
        .arg(&replay_file)
        .arg(release_gate_workflow())
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");
    server.join().unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("workflow decision denied by step `release_decision` with value `NO_GO`")
    );
    let replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_file).unwrap()).unwrap();
    assert_eq!(replay["decision"]["value"], "NO_GO");
    assert_eq!(replay["decision"]["outcome"], "deny");
}

#[test]
fn run_command_rejects_decision_enforcement_without_a_policy() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--enforce-decision")
        .arg(support_triage_workflow())
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("decision enforcement requires a `[decision]` workflow policy")
    );
}

#[test]
fn run_command_renders_literal_workflow_inputs() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--input")
        .arg("git_diff=timeout_seconds changed from 30 to 0")
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["workflow_name"], "git-change-review");
    assert_eq!(report["steps"].as_array().unwrap().len(), 2);
}

#[test]
fn run_command_renders_file_workflow_inputs() {
    let input_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("workflow-input.diff");
    fs::create_dir_all(input_file.parent().unwrap()).unwrap();
    fs::write(&input_file, "timeout_seconds changed from 30 to 0").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--input-file")
        .arg(format!("git_diff={}", input_file.display()))
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_command_rejects_missing_and_unused_workflow_inputs() {
    let missing = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr)
            .contains("workflow input `git_diff` is required but was not supplied")
    );

    let unused = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--input")
        .arg("unused=value")
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(!unused.status.success());
    assert!(
        String::from_utf8_lossy(&unused.stderr)
            .contains("workflow input `unused` was supplied but is not used")
    );
}

#[test]
fn run_command_injects_current_git_diff() {
    let repository = create_git_diff_repository("workflow-git-diff");
    fs::write(repository.join("service.toml"), "timeout_seconds = 0\n").unwrap();

    let first = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--git-diff")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_report: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("stdout should be JSON");

    fs::write(repository.join("service.toml"), "timeout_seconds = 5\n").unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--git-diff")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_report: serde_json::Value =
        serde_json::from_slice(&second.stdout).expect("stdout should be JSON");

    assert_ne!(
        first_report["steps"][0]["input_hash"],
        second_report["steps"][0]["input_hash"]
    );
}

#[test]
fn run_command_injects_git_diff_from_base_revision() {
    let repository = create_git_diff_repository("workflow-git-base-diff");
    fs::write(repository.join("service.toml"), "timeout_seconds = 0\n").unwrap();
    for arguments in [
        vec!["add", "service.toml"],
        vec!["commit", "-m", "Remove timeout"],
    ] {
        let output = Command::new("git")
            .args(arguments)
            .current_dir(&repository)
            .output()
            .expect("Git should execute");
        assert!(
            output.status.success(),
            "Git stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--git-diff-base")
        .arg("HEAD~1")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_command_rejects_empty_and_duplicate_git_diff_inputs() {
    let repository = create_git_diff_repository("workflow-empty-git-diff");
    let empty = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--git-diff")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(!empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stderr).contains("contains no tracked changes"));

    fs::write(repository.join("service.toml"), "timeout_seconds = 0\n").unwrap();
    let duplicate = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--input")
        .arg("git_diff=manual diff")
        .arg("--git-diff")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");
    assert!(!duplicate.status.success());
    assert!(
        String::from_utf8_lossy(&duplicate.stderr)
            .contains("workflow input `git_diff` was supplied more than once")
    );
}

#[test]
fn run_command_rejects_git_diffs_over_one_mebibyte() {
    let repository = create_git_diff_repository("workflow-oversized-git-diff");
    fs::write(
        repository.join("service.toml"),
        format!("payload = \"{}\"\n", "x".repeat(1024 * 1024)),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--git-diff")
        .arg("--repository")
        .arg(&repository)
        .arg(git_change_review_workflow())
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("exceeding the 1 MiB limit"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
fn run_command_redacts_values_from_environment_variables() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact-env")
        .arg("classification=VOGON_TEST_REDACTION")
        .arg(fixture)
        .env("VOGON_TEST_REDACTION", classification_output())
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
fn run_command_reports_missing_redaction_environment_variables() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact-env")
        .arg("classification=VOGON_MISSING_REDACTION")
        .arg(fixture)
        .env_remove("VOGON_MISSING_REDACTION")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("redaction environment variable `VOGON_MISSING_REDACTION` is not set"));
    assert!(!stderr.contains(classification_output()));
}

#[test]
fn run_command_rejects_duplicate_literal_and_environment_redaction_labels() {
    let fixture = support_triage_workflow();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--redact")
        .arg("classification=first")
        .arg("--redact-env")
        .arg("classification=VOGON_TEST_REDACTION")
        .arg(fixture)
        .env("VOGON_TEST_REDACTION", "second")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("duplicate redaction label `classification`"));
    assert!(!stderr.contains("second"));
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
fn run_and_verify_support_unauthenticated_openai_compatible_endpoint() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let mut requests = Vec::new();
        for _ in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            requests.push(read_http_request(&mut stream));
            let body = r#"{"choices":[{"message":{"content":"local response"}}]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
        requests
    });
    let fixture = support_triage_workflow();
    let replay_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("unauthenticated-openai-compatible.replay.json");
    remove_file_if_exists(&replay_file);
    let base_url = format!("http://{address}");

    let run = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-base-url")
        .arg(&base_url)
        .arg("--openai-compatible-model")
        .arg("local/model")
        .arg("--openai-compatible-no-auth")
        .arg("--openai-compatible-max-retries")
        .arg("0")
        .arg("--output")
        .arg(&replay_file)
        .arg(&fixture)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let verify = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg(&fixture)
        .arg(&replay_file)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("verify command should execute");
    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let requests = server.join().unwrap();
    let replay: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(replay_file).unwrap()).unwrap();
    assert_eq!(replay["runtime"]["parameters"]["auth_mode"], "none");
    assert!(
        replay["runtime"]["cache_identity"]
            .as_str()
            .unwrap()
            .contains(":auth=none:")
    );
    assert!(requests.iter().all(|request| {
        !String::from_utf8_lossy(request)
            .to_ascii_lowercase()
            .contains("\r\nauthorization:")
    }));
}

#[test]
fn verify_command_reuses_run_cache_without_provider_access() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            read_http_request(&mut stream);
            let body = r#"{"choices":[{"message":{"content":"cached response"}}]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });
    let fixture = support_triage_workflow();
    let artifact_directory = repo_root().join("target").join("vogon-tests");
    let replay_file = artifact_directory.join("cached-provider.replay.json");
    let cache_file = artifact_directory.join("cached-provider.cache.json");
    remove_file_if_exists(&replay_file);
    remove_file_if_exists(&cache_file);
    let base_url = format!("http://{address}");

    let run = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-base-url")
        .arg(&base_url)
        .arg("--openai-compatible-model")
        .arg("local/model")
        .arg("--openai-compatible-no-auth")
        .arg("--openai-compatible-max-retries")
        .arg("0")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg("--output")
        .arg(&replay_file)
        .arg(&fixture)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");
    assert!(
        run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    server.join().unwrap();

    let verify = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("verify")
        .arg("--cache-file")
        .arg(&cache_file)
        .arg(&fixture)
        .arg(&replay_file)
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("verify command should execute");

    assert!(
        verify.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains("Replay verified"));
}

#[test]
fn run_command_rejects_remote_plaintext_openai_compatible_endpoint() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("openai-compatible")
        .arg("--openai-compatible-base-url")
        .arg("http://example.test/v1")
        .arg("--openai-compatible-no-auth")
        .arg(support_triage_workflow())
        .env_remove("OPENAI_COMPATIBLE_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("remote base URLs must use HTTPS"));
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
fn run_command_reports_missing_nvidia_api_key() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("nvidia")
        .arg(support_triage_workflow())
        .env_remove("NVIDIA_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NVIDIA_API_KEY must be set"));
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
fn run_command_rejects_zero_nvidia_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("nvidia")
        .arg("--nvidia-timeout-seconds")
        .arg("0")
        .arg(support_triage_workflow())
        .env_remove("NVIDIA_API_KEY")
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
fn run_command_rejects_excessive_nvidia_retry_count() {
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--provider")
        .arg("nvidia")
        .arg("--nvidia-max-retries")
        .arg("21")
        .arg(support_triage_workflow())
        .env_remove("NVIDIA_API_KEY")
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--nvidia-max-retries must be between 0 and 20"));
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
fn run_command_rejects_overlapping_output_and_cache_paths() {
    let fixture = support_triage_workflow();
    let artifact_file = repo_root()
        .join("target")
        .join("vogon-tests")
        .join("overlapping-run-artifact.json");
    remove_file_if_exists(&artifact_file);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("run")
        .arg("--output")
        .arg(&artifact_file)
        .arg("--cache-file")
        .arg(&artifact_file)
        .arg(fixture)
        .output()
        .expect("run command should execute");

    assert!(!output.status.success());
    assert!(!artifact_file.exists());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("replay output path"));
    assert!(stderr.contains("run cache path"));
    assert!(stderr.contains("must be different"));
    assert!(stderr.contains(&artifact_file.display().to_string()));
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

fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0; 1024];

    loop {
        let bytes_read = stream.read(&mut chunk).unwrap();
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);

        let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let content_length = String::from_utf8_lossy(&buffer[..header_end])
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        if buffer.len() >= header_end + 4 + content_length {
            break;
        }
    }

    buffer
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
