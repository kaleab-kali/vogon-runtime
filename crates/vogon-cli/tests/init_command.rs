use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn test_output_path(file_name: &str) -> PathBuf {
    repo_root()
        .join("target")
        .join("vogon-tests")
        .join("init-command")
        .join(file_name)
}

fn remove_file_if_exists(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to remove {}: {error}", path.display()),
    }
}

#[test]
fn init_command_writes_a_valid_starter_workflow() {
    let workflow_path = test_output_path("starter.toml");
    remove_file_if_exists(&workflow_path);

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("init")
        .arg("--output")
        .arg(&workflow_path)
        .output()
        .expect("init command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Created workflow file"),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let check_output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("check")
        .arg("--json")
        .arg(&workflow_path)
        .output()
        .expect("check command should execute");

    assert!(
        check_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&check_output.stderr)
    );
    let summary: serde_json::Value =
        serde_json::from_slice(&check_output.stdout).expect("stdout should be JSON");
    assert_eq!(summary["workflow_name"], "starter-workflow");
    assert_eq!(summary["step_count"], 2);
}

#[test]
fn init_command_refuses_to_overwrite_existing_workflow() {
    let workflow_path = test_output_path("existing.toml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, "name = \"existing\"\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("init")
        .arg("--output")
        .arg(&workflow_path)
        .output()
        .expect("init command should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("already exists; pass --force"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&workflow_path).unwrap(),
        "name = \"existing\"\n"
    );
}

#[test]
fn init_command_can_force_overwrite_existing_workflow() {
    let workflow_path = test_output_path("force.toml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, "name = \"existing\"\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vogon"))
        .arg("init")
        .arg("--force")
        .arg("--output")
        .arg(&workflow_path)
        .output()
        .expect("init command should execute");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let workflow = fs::read_to_string(&workflow_path).unwrap();
    assert!(workflow.contains("name = \"starter-workflow\""));
    assert!(workflow.contains("id = \"draft\""));
    assert!(workflow.contains("id = \"review\""));
}
