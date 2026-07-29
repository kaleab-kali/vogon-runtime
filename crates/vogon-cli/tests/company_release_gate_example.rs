use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example_root() -> PathBuf {
    repo_root().join("examples").join("company-release-gate")
}

fn run_git(repository: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()
        .expect("Git should execute");
    assert!(
        output.status.success(),
        "Git {:?} failed: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_vogon(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vogon"))
        .args(arguments)
        .output()
        .expect("Vogon should execute")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_company_example_runs_against_an_external_git_repository() {
    let external = repo_root()
        .join("target")
        .join("vogon-tests")
        .join(format!("company-release-example-{}", std::process::id()));
    if external.exists() {
        fs::remove_dir_all(&external).unwrap();
    }
    fs::create_dir_all(external.join("service")).unwrap();
    fs::copy(
        example_root().join("baseline").join("deployment.toml"),
        external.join("service").join("deployment.toml"),
    )
    .unwrap();

    run_git(&external, &["init", "-b", "main"]);
    run_git(&external, &["config", "user.name", "Vogon Example"]);
    run_git(
        &external,
        &["config", "user.email", "vogon-example@example.invalid"],
    );
    run_git(&external, &["add", "service/deployment.toml"]);
    run_git(&external, &["commit", "-m", "Add safe deployment baseline"]);

    fs::copy(
        example_root().join("candidate").join("deployment.toml"),
        external.join("service").join("deployment.toml"),
    )
    .unwrap();
    let diff = Command::new("git")
        .args(["diff", "--", "service/deployment.toml"])
        .current_dir(&external)
        .output()
        .unwrap();
    assert_success(&diff);
    let diff = String::from_utf8(diff.stdout).unwrap();
    assert!(diff.contains("health_check_enabled = false"));
    assert!(diff.contains("rollback_on_failure = false"));
    assert!(diff.contains("minimum_healthy_instances = 0"));
    fs::create_dir_all(external.join(".vogon")).unwrap();

    let workflow = example_root().join("workflows").join("context-smoke.toml");
    let release_workflow = example_root().join("workflows").join("release-gate.toml");
    let replay = external.join(".vogon").join("context.replay.json");
    let cached_replay = external.join(".vogon").join("cached.replay.json");
    let cache = external.join(".vogon").join("context.cache.json");
    let external_text = external.to_string_lossy();
    let workflow_text = workflow.to_string_lossy();
    let replay_text = replay.to_string_lossy();
    let cached_replay_text = cached_replay.to_string_lossy();
    let cache_text = cache.to_string_lossy();

    let first = run_vogon(&[
        "run",
        "--provider",
        "deterministic",
        "--git-diff",
        "--repository",
        &external_text,
        "--input",
        "service_owner=payments-platform",
        "--cache-file",
        &cache_text,
        "--output",
        &replay_text,
        &workflow_text,
    ]);
    assert_success(&first);

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&replay).unwrap()).unwrap();
    assert_eq!(report["workflow_name"], "company-release-context-smoke");
    assert_eq!(report["runtime"]["provider"], "deterministic");
    assert_eq!(report["runtime"]["model"], "deterministic-echo");
    assert_eq!(report["steps"].as_array().unwrap().len(), 1);
    assert_eq!(report["execution_policy_hash"].as_str().unwrap().len(), 64);
    let cache_data: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cache).unwrap()).unwrap();
    assert_eq!(cache_data["outputs"].as_object().unwrap().len(), 1);
    assert_eq!(cache_data["insertion_order"].as_array().unwrap().len(), 1);

    let second = run_vogon(&[
        "run",
        "--provider",
        "deterministic",
        "--git-diff",
        "--repository",
        &external_text,
        "--input",
        "service_owner=payments-platform",
        "--cache-file",
        &cache_text,
        "--output",
        &cached_replay_text,
        &workflow_text,
    ]);
    assert_success(&second);
    assert_eq!(
        fs::read_to_string(&replay).unwrap(),
        fs::read_to_string(&cached_replay).unwrap()
    );

    for mode in ["exact", "structure"] {
        let verification = run_vogon(&[
            "verify",
            "--git-diff",
            "--repository",
            &external_text,
            "--input",
            "service_owner=payments-platform",
            "--cache-file",
            &cache_text,
            "--mode",
            mode,
            &workflow_text,
            &replay_text,
        ]);
        assert_success(&verification);
    }

    let check = run_vogon(&["check", "--json", &release_workflow.to_string_lossy()]);
    assert_success(&check);
    let summary: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(
        summary["required_inputs"],
        serde_json::json!(["git_diff", "service_owner"])
    );
    assert_eq!(summary["decision"]["deny"], serde_json::json!(["NO_GO"]));
    assert_eq!(
        summary["execution"]["allowed_providers"],
        serde_json::json!(["nvidia"])
    );

    fs::remove_dir_all(external).unwrap();
}
