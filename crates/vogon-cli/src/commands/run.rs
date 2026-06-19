use std::{fs, io, path::Path, process};

use clap::ValueEnum;
use vogon_adapters::DeterministicEchoModel;
#[cfg(feature = "gemini")]
use vogon_adapters::GeminiModel;
use vogon_core::{ModelAdapter, RedactionSet, RunReport, Runtime};

use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

pub fn run(
    workflow_file: &Path,
    redaction_values: &[String],
    output: Option<&Path>,
    model_config: RunModelConfig<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let redactions = parse_redactions(redaction_values)?;
    let report = run_with_model(&workflow, &redactions, model_config)?;
    let replay_json = format!("{}\n", serde_json::to_string_pretty(&report)?);

    if let Some(output) = output {
        create_output_parent(output)?;
        reject_directory_output(output)?;

        write_replay_file(output, &replay_json).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to write replay output `{}`: {error}",
                    output.display()
                ),
            )
        })?;
        println!("Replay written: {}", output.display());
    } else {
        print!("{replay_json}");
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ModelProvider {
    Deterministic,
    Gemini,
}

#[derive(Debug, Clone, Copy)]
pub struct RunModelConfig<'a> {
    pub provider: ModelProvider,
    pub gemini_model: &'a str,
}

fn run_with_model(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model_config: RunModelConfig<'_>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    match model_config.provider {
        ModelProvider::Deterministic => {
            run_with_adapter(DeterministicEchoModel, workflow, redactions)
        }
        ModelProvider::Gemini => run_with_gemini(workflow, redactions, model_config.gemini_model),
    }
}

fn run_with_adapter<A>(
    adapter: A,
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
) -> Result<RunReport, Box<dyn std::error::Error>>
where
    A: ModelAdapter,
{
    Ok(Runtime::new(adapter).run_with_redactions(workflow, redactions)?)
}

#[cfg(feature = "gemini")]
fn run_with_gemini(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(GeminiModel::from_env(model)?, workflow, redactions)
}

#[cfg(not(feature = "gemini"))]
fn run_with_gemini(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Gemini provider support is not enabled in this build",
    )
    .into())
}

fn reject_directory_output(output: &Path) -> io::Result<()> {
    if output.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("replay output path `{}` is a directory", output.display()),
        ));
    }

    Ok(())
}

fn create_output_parent(output: &Path) -> io::Result<()> {
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to create replay output directory `{}`: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    Ok(())
}

fn write_replay_file(output: &Path, replay_json: &str) -> io::Result<()> {
    let temp_output = temp_output_path(output)?;
    fs::write(&temp_output, replay_json)?;

    match fs::rename(&temp_output, output) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(output)?;
            fs::rename(&temp_output, output)
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_output);
            Err(error)
        }
    }
}

fn temp_output_path(output: &Path) -> io::Result<std::path::PathBuf> {
    let file_name = output.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("output path `{}` has no file name", output.display()),
        )
    })?;
    let temp_file_name = format!(".{}.{}.tmp", file_name.to_string_lossy(), process::id());

    Ok(output.with_file_name(temp_file_name))
}
