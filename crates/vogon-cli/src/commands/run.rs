#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
use std::time::Duration;
use std::{fs, io, path::Path, process};

use clap::ValueEnum;
use vogon_adapters::DeterministicEchoModel;
#[cfg(feature = "gemini")]
use vogon_adapters::GeminiModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::GroqModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::HuggingFaceModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::OpenAiCompatibleModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::OpenRouterModel;
#[cfg(feature = "gemini")]
pub use vogon_adapters::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_TIMEOUT_SECONDS, MAX_GEMINI_RETRIES,
};
#[cfg(feature = "openai-compatible")]
pub use vogon_adapters::{
    DEFAULT_GROQ_BASE_URL, DEFAULT_GROQ_MAX_RETRIES, DEFAULT_GROQ_MODEL,
    DEFAULT_GROQ_TIMEOUT_SECONDS, DEFAULT_HUGGING_FACE_BASE_URL, DEFAULT_HUGGING_FACE_MAX_RETRIES,
    DEFAULT_HUGGING_FACE_MODEL, DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS,
    DEFAULT_OPENAI_COMPATIBLE_BASE_URL, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
    DEFAULT_OPENAI_COMPATIBLE_MODEL, DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS,
    DEFAULT_OPENROUTER_BASE_URL, DEFAULT_OPENROUTER_MAX_RETRIES, DEFAULT_OPENROUTER_MODEL,
    DEFAULT_OPENROUTER_TIMEOUT_SECONDS, MAX_GROQ_RETRIES, MAX_HUGGING_FACE_RETRIES,
    MAX_OPENAI_COMPATIBLE_RETRIES, MAX_OPENROUTER_RETRIES,
};
use vogon_core::{ModelAdapter, RedactionSet, RunReport, Runtime};

use crate::commands::redaction::parse_redactions;
use crate::commands::workflow_file::read_toml_workflow;

#[cfg(not(feature = "gemini"))]
pub const DEFAULT_GEMINI_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "gemini"))]
pub const MAX_GEMINI_RETRIES: u32 = 20;
#[cfg(not(feature = "gemini"))]
pub const DEFAULT_GEMINI_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENAI_COMPATIBLE_BASE_URL: &str = "https://router.huggingface.co/v1";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "openai-compatible"))]
pub const MAX_OPENAI_COMPATIBLE_RETRIES: u32 = 20;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENAI_COMPATIBLE_MODEL: &str = "openai/gpt-oss-120b:fastest";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_GROQ_MODEL: &str = "llama-3.1-8b-instant";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_GROQ_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_GROQ_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "openai-compatible"))]
pub const MAX_GROQ_RETRIES: u32 = 20;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_HUGGING_FACE_BASE_URL: &str = "https://router.huggingface.co/v1";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_HUGGING_FACE_MODEL: &str = "openai/gpt-oss-120b:fastest";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_HUGGING_FACE_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "openai-compatible"))]
pub const MAX_HUGGING_FACE_RETRIES: u32 = 20;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENROUTER_MODEL: &str = "openrouter/free";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENROUTER_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_OPENROUTER_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "openai-compatible"))]
pub const MAX_OPENROUTER_RETRIES: u32 = 20;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ModelProvider {
    Deterministic,
    Gemini,
    Groq,
    #[value(name = "hugging-face")]
    HuggingFace,
    #[value(name = "openai-compatible")]
    OpenAiCompatible,
    #[value(name = "openrouter")]
    OpenRouter,
}

impl ModelProvider {
    pub fn from_runtime_provider_name(provider: &str) -> Option<Self> {
        match provider {
            "deterministic" | "legacy" => Some(Self::Deterministic),
            "gemini" => Some(Self::Gemini),
            "groq" => Some(Self::Groq),
            "hugging-face" => Some(Self::HuggingFace),
            "openai-compatible" => Some(Self::OpenAiCompatible),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RunModelConfig<'a> {
    pub provider: ModelProvider,
    pub gemini_model: &'a str,
    pub gemini_timeout_seconds: u64,
    pub gemini_max_retries: u32,
    pub groq_model: &'a str,
    pub groq_timeout_seconds: u64,
    pub groq_max_retries: u32,
    pub hugging_face_model: &'a str,
    pub hugging_face_timeout_seconds: u64,
    pub hugging_face_max_retries: u32,
    pub openrouter_model: &'a str,
    pub openrouter_timeout_seconds: u64,
    pub openrouter_max_retries: u32,
    pub openai_compatible_base_url: &'a str,
    pub openai_compatible_model: &'a str,
    pub openai_compatible_timeout_seconds: u64,
    pub openai_compatible_max_retries: u32,
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
        ModelProvider::Gemini => run_with_gemini(
            workflow,
            redactions,
            model_config.gemini_model,
            model_config.gemini_timeout_seconds,
            model_config.gemini_max_retries,
        ),
        ModelProvider::Groq => run_with_groq(
            workflow,
            redactions,
            model_config.groq_model,
            model_config.groq_timeout_seconds,
            model_config.groq_max_retries,
        ),
        ModelProvider::HuggingFace => run_with_hugging_face(
            workflow,
            redactions,
            model_config.hugging_face_model,
            model_config.hugging_face_timeout_seconds,
            model_config.hugging_face_max_retries,
        ),
        ModelProvider::OpenRouter => run_with_openrouter(
            workflow,
            redactions,
            model_config.openrouter_model,
            model_config.openrouter_timeout_seconds,
            model_config.openrouter_max_retries,
        ),
        ModelProvider::OpenAiCompatible => run_with_openai_compatible(
            workflow,
            redactions,
            model_config.openai_compatible_base_url,
            model_config.openai_compatible_model,
            model_config.openai_compatible_timeout_seconds,
            model_config.openai_compatible_max_retries,
        ),
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
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        GeminiModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
    )
}

#[cfg(not(feature = "gemini"))]
fn run_with_gemini(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Gemini provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn run_with_groq(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        GroqModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_groq(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Groq provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn run_with_hugging_face(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        HuggingFaceModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_hugging_face(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Hugging Face provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn run_with_openrouter(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        OpenRouterModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_openrouter(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "OpenRouter provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn run_with_openai_compatible(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    base_url: &str,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        OpenAiCompatibleModel::from_env_with_base_url_model_timeout_and_retries(
            base_url,
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_openai_compatible(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _base_url: &str,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "OpenAI-compatible provider support is not enabled in this build",
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
