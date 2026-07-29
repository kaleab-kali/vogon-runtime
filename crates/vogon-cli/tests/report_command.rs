use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use vogon_core::{
    DecisionPolicy, ExecutionPolicy, ModelAdapter, Result, Runtime, RuntimeMetadata, Step, StepId,
    Workflow,
};

#[derive(Debug, Clone, Copy)]
struct ReleaseDecisionModel;

impl ModelAdapter for ReleaseDecisionModel {
    fn complete(&self, _step: &Step, _input: &str) -> Result<String> {
        Ok(
            r#"{"decision":"NO_GO","reasons":["health check disabled","<script>alert(\"x\")</script>"],"required_actions":["restore internal-host rollback"]}"#
                .to_owned(),
        )
    }

    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new("nvidia", "test-adapter", "1", "test-cache")
            .with_model("meta/test-model")
    }
}

fn output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("vogon-tests")
        .join(format!("report-command-{}", std::process::id()))
}

fn remove_path(path: &Path) {
    if path.is_dir() {
        fs::remove_dir_all(path).unwrap();
    } else if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

fn decision_workflow() -> Workflow {
    let step_id = StepId::new("release_decision").unwrap();
    Workflow::new(
        "company-release-gate",
        vec![Step::new(step_id.clone(), "Decide")],
    )
    .unwrap()
    .with_decision(DecisionPolicy {
        step: step_id,
        pointer: "/decision".to_owned(),
        allow: vec!["GO".to_owned()],
        deny: vec!["NO_GO".to_owned()],
    })
    .unwrap()
    .with_execution_policy(ExecutionPolicy {
        allowed_providers: vec!["nvidia".to_owned()],
        allowed_models: vec!["meta/test-model".to_owned()],
        max_step_output_bytes: Some(4096),
    })
    .unwrap()
}

#[test]
fn report_command_renders_escaped_redacted_decision_evidence_and_rejects_tampering() {
    let root = output_root();
    remove_path(&root);
    fs::create_dir_all(&root).unwrap();
    let replay_path = root.join("release.replay.json");
    let report_path = root.join("nested").join("release-report.html");
    let report = Runtime::new(ReleaseDecisionModel)
        .run(&decision_workflow())
        .unwrap();
    fs::write(&replay_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("report")
        .arg("--redact")
        .arg("host=internal-host")
        .arg("--output")
        .arg(&report_path)
        .arg(&replay_path)
        .output()
        .expect("report command should execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("HTML evidence report written:"));

    let html = fs::read_to_string(&report_path).unwrap();
    for expected in [
        "VOGON RUNTIME",
        "company-release-gate",
        "RELEASE BLOCKED",
        "NO_GO",
        "health check disabled",
        "restore [REDACTED:host] rollback",
        "Self-consistency check passed",
        "Content-Security-Policy",
        "&lt;script&gt;alert(\\&quot;x\\&quot;)&lt;/script&gt;",
    ] {
        assert!(
            html.contains(expected),
            "report should contain `{expected}`"
        );
    }
    assert!(!html.contains("internal-host"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("src=\"http"));

    let replay_before = fs::read_to_string(&replay_path).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("report")
        .arg("--output")
        .arg(&replay_path)
        .arg(&replay_path)
        .output()
        .expect("report command should execute");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must differ from the source replay"));
    assert_eq!(fs::read_to_string(&replay_path).unwrap(), replay_before);

    let mut tampered = report;
    tampered.steps[0].output.push_str(" changed");
    let tampered_replay = root.join("tampered.replay.json");
    let tampered_report = root.join("tampered-report.html");
    fs::write(
        &tampered_replay,
        serde_json::to_string_pretty(&tampered).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("report")
        .arg("--output")
        .arg(&tampered_report)
        .arg(&tampered_replay)
        .output()
        .expect("report command should execute");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("replay integrity check failed"));
    assert!(!tampered_report.exists());

    fs::remove_dir_all(root).unwrap();
}
