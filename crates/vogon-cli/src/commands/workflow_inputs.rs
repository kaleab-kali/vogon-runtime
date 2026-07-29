use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use clap::Args;
use vogon_core::Workflow;

use crate::commands::file_io;

const MAX_WORKFLOW_INPUT_BYTES: usize = 1024 * 1024;
const GIT_DIFF_INPUT_NAME: &str = "git_diff";

#[derive(Debug, Clone, Args)]
pub struct WorkflowInputArgs {
    /// Supply a named workflow input literal. May be repeated.
    #[arg(long = "input", value_name = "NAME=VALUE")]
    pub values: Vec<String>,

    /// Read a named workflow input from a UTF-8 file. May be repeated.
    #[arg(long = "input-file", value_name = "NAME=FILE")]
    pub files: Vec<String>,

    /// Inject tracked staged and unstaged changes as `git_diff`.
    #[arg(long, conflicts_with = "git_diff_base")]
    pub git_diff: bool,

    /// Inject `REVISION...HEAD` as `git_diff`, for pull request CI.
    #[arg(long, value_name = "REVISION", conflicts_with = "git_diff")]
    pub git_diff_base: Option<String>,

    /// Git repository used by `--git-diff` or `--git-diff-base`.
    #[arg(long, value_name = "DIRECTORY", default_value = ".")]
    pub repository: PathBuf,
}

pub fn render_workflow(
    workflow: &Workflow,
    args: &WorkflowInputArgs,
) -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut inputs = BTreeMap::new();

    for assignment in &args.values {
        let (name, value) = parse_assignment(assignment, "--input")?;
        insert_input(&mut inputs, name, value.to_owned())?;
    }

    for assignment in &args.files {
        let (name, path) = parse_assignment(assignment, "--input-file")?;
        if path.is_empty() {
            return Err(invalid_input("--input-file path cannot be empty"));
        }
        let value = file_io::read_to_string(Path::new(path), "workflow input file")?;
        insert_input(&mut inputs, name, value)?;
    }

    if args.git_diff || args.git_diff_base.is_some() {
        let diff = read_git_diff(&args.repository, args.git_diff_base.as_deref())?;
        insert_input(&mut inputs, GIT_DIFF_INPUT_NAME, diff)?;
    }

    let input_bytes = inputs.values().map(String::len).sum::<usize>();
    if input_bytes > MAX_WORKFLOW_INPUT_BYTES {
        return Err(invalid_input(format!(
            "workflow inputs total {input_bytes} bytes, exceeding the 1 MiB limit"
        )));
    }

    Ok(workflow.render_inputs(&inputs)?)
}

fn parse_assignment<'a>(
    assignment: &'a str,
    option: &str,
) -> Result<(&'a str, &'a str), Box<dyn std::error::Error>> {
    let Some((name, value)) = assignment.split_once('=') else {
        return Err(invalid_input(format!(
            "{option} must use NAME=VALUE syntax"
        )));
    };
    if name.is_empty() {
        return Err(invalid_input(format!("{option} name cannot be empty")));
    }
    Ok((name, value))
}

fn insert_input(
    inputs: &mut BTreeMap<String, String>,
    name: &str,
    value: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if inputs.insert(name.to_owned(), value).is_some() {
        return Err(invalid_input(format!(
            "workflow input `{name}` was supplied more than once"
        )));
    }
    Ok(())
}

fn read_git_diff(
    repository: &Path,
    base: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(base) = base {
        if base.is_empty() || base.starts_with('-') {
            return Err(invalid_input(
                "--git-diff-base must be a non-option Git revision",
            ));
        }
    }

    let mut command = Command::new("git");
    command
        .current_dir(repository)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args([
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--ignore-submodules=all",
        ]);

    if let Some(base) = base {
        command.arg(format!("{base}...HEAD"));
    } else {
        command.arg("HEAD");
    }
    command.arg("--");

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to execute Git in repository `{}`: {error}",
                    repository.display()
                ),
            )
        })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("failed to capture Git standard output"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("failed to capture Git standard error"))?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, MAX_WORKFLOW_INPUT_BYTES + 1));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, 4096));
    let status = child.wait().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed while waiting for Git in repository `{}`: {error}",
                repository.display()
            ),
        )
    })?;
    let (stdout, stdout_bytes) = join_reader(stdout_reader, "standard output")?;
    let (stderr, _) = join_reader(stderr_reader, "standard error")?;

    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr);
        return Err(io::Error::other(format!(
            "Git diff failed in repository `{}`: {}",
            repository.display(),
            bounded_message(&stderr)
        ))
        .into());
    }
    if stdout_bytes > MAX_WORKFLOW_INPUT_BYTES {
        return Err(invalid_input(format!(
            "Git diff is {stdout_bytes} bytes, exceeding the 1 MiB limit"
        )));
    }

    let diff = String::from_utf8(stdout).map_err(|error| {
        invalid_input(format!(
            "Git diff in repository `{}` is not valid UTF-8: {error}",
            repository.display()
        ))
    })?;
    if diff.trim().is_empty() {
        return Err(invalid_input(format!(
            "Git diff in repository `{}` contains no tracked changes",
            repository.display()
        )));
    }

    Ok(diff)
}

fn read_bounded(mut reader: impl Read, retained_bytes: usize) -> io::Result<(Vec<u8>, usize)> {
    let mut retained = Vec::with_capacity(retained_bytes);
    let mut total = 0usize;
    let mut buffer = [0_u8; 8192];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read);
        let remaining = retained_bytes.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..read.min(remaining)]);
    }

    Ok((retained, total))
}

fn join_reader(
    reader: thread::JoinHandle<io::Result<(Vec<u8>, usize)>>,
    stream_name: &str,
) -> io::Result<(Vec<u8>, usize)> {
    reader
        .join()
        .map_err(|_| io::Error::other(format!("Git {stream_name} reader panicked")))?
}

fn bounded_message(message: &str) -> String {
    const MAX_CHARS: usize = 1000;
    let mut bounded = message.chars().take(MAX_CHARS).collect::<String>();
    if message.chars().count() > MAX_CHARS {
        bounded.push_str("...");
    }
    bounded.trim().to_owned()
}

fn invalid_input(message: impl Into<String>) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidInput, message.into()).into()
}
