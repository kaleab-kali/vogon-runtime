#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
use std::time::Duration;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
    process,
};

use clap::ValueEnum;
use vogon_adapters::DeterministicEchoModel;
#[cfg(feature = "gemini")]
use vogon_adapters::GeminiModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::GroqModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::HuggingFaceModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::NvidiaModel;
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
    DEFAULT_HUGGING_FACE_MODEL, DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS, DEFAULT_NVIDIA_BASE_URL,
    DEFAULT_NVIDIA_MAX_RETRIES, DEFAULT_NVIDIA_MODEL, DEFAULT_NVIDIA_TIMEOUT_SECONDS,
    DEFAULT_OPENAI_COMPATIBLE_BASE_URL, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
    DEFAULT_OPENAI_COMPATIBLE_MODEL, DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS,
    DEFAULT_OPENROUTER_BASE_URL, DEFAULT_OPENROUTER_MAX_RETRIES, DEFAULT_OPENROUTER_MODEL,
    DEFAULT_OPENROUTER_TIMEOUT_SECONDS, MAX_GROQ_RETRIES, MAX_HUGGING_FACE_RETRIES,
    MAX_NVIDIA_RETRIES, MAX_OPENAI_COMPATIBLE_RETRIES, MAX_OPENROUTER_RETRIES,
};
use vogon_core::{ModelAdapter, RedactionSet, RunCache, RunReport, Runtime};

use crate::commands::file_io;
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
pub const DEFAULT_NVIDIA_BASE_URL: &str = "https://integrate.api.nvidia.com/v1";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_NVIDIA_MODEL: &str = "meta/llama-3.1-8b-instruct";
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_NVIDIA_TIMEOUT_SECONDS: u64 = 30;
#[cfg(not(feature = "openai-compatible"))]
pub const DEFAULT_NVIDIA_MAX_RETRIES: u32 = 2;
#[cfg(not(feature = "openai-compatible"))]
pub const MAX_NVIDIA_RETRIES: u32 = 20;
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
    cache_file: Option<&Path>,
    cache_max_entries: usize,
    model_config: RunModelConfig<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    reject_overlapping_artifact_paths(output, cache_file)?;
    let workflow = read_toml_workflow(workflow_file)?;
    let redactions = parse_redactions(redaction_values)?;
    let mut cache = load_run_cache(cache_file, cache_max_entries)?;
    let report = run_with_model(&workflow, &redactions, model_config, cache.as_mut())?;
    let replay_json = format!("{}\n", serde_json::to_string_pretty(&report)?);
    if let (Some(cache_file), Some(cache)) = (cache_file, cache.as_ref()) {
        write_run_cache_file(cache_file, cache)?;
    }

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
    Nvidia,
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
            "nvidia" => Some(Self::Nvidia),
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
    pub nvidia_model: &'a str,
    pub nvidia_timeout_seconds: u64,
    pub nvidia_max_retries: u32,
    pub openrouter_model: &'a str,
    pub openrouter_timeout_seconds: u64,
    pub openrouter_max_retries: u32,
    pub openai_compatible_base_url: &'a str,
    pub openai_compatible_model: &'a str,
    pub openai_compatible_no_auth: bool,
    pub openai_compatible_timeout_seconds: u64,
    pub openai_compatible_max_retries: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct OpenAiCompatibleConfig<'a> {
    pub base_url: &'a str,
    pub model: &'a str,
    pub no_auth: bool,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

fn run_with_model(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model_config: RunModelConfig<'_>,
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    match model_config.provider {
        ModelProvider::Deterministic => {
            run_with_adapter(DeterministicEchoModel, workflow, redactions, cache)
        }
        ModelProvider::Gemini => run_with_gemini(
            workflow,
            redactions,
            model_config.gemini_model,
            model_config.gemini_timeout_seconds,
            model_config.gemini_max_retries,
            cache,
        ),
        ModelProvider::Groq => run_with_groq(
            workflow,
            redactions,
            model_config.groq_model,
            model_config.groq_timeout_seconds,
            model_config.groq_max_retries,
            cache,
        ),
        ModelProvider::HuggingFace => run_with_hugging_face(
            workflow,
            redactions,
            model_config.hugging_face_model,
            model_config.hugging_face_timeout_seconds,
            model_config.hugging_face_max_retries,
            cache,
        ),
        ModelProvider::Nvidia => run_with_nvidia(
            workflow,
            redactions,
            model_config.nvidia_model,
            model_config.nvidia_timeout_seconds,
            model_config.nvidia_max_retries,
            cache,
        ),
        ModelProvider::OpenRouter => run_with_openrouter(
            workflow,
            redactions,
            model_config.openrouter_model,
            model_config.openrouter_timeout_seconds,
            model_config.openrouter_max_retries,
            cache,
        ),
        ModelProvider::OpenAiCompatible => run_with_openai_compatible(
            workflow,
            redactions,
            OpenAiCompatibleConfig {
                base_url: model_config.openai_compatible_base_url,
                model: model_config.openai_compatible_model,
                no_auth: model_config.openai_compatible_no_auth,
                timeout_seconds: model_config.openai_compatible_timeout_seconds,
                max_retries: model_config.openai_compatible_max_retries,
            },
            cache,
        ),
    }
}

fn run_with_adapter<A>(
    adapter: A,
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>>
where
    A: ModelAdapter,
{
    let runtime = Runtime::new(adapter);
    match cache {
        Some(cache) => Ok(runtime.run_with_cache_and_redactions(workflow, cache, redactions)?),
        None => Ok(runtime.run_with_redactions(workflow, redactions)?),
    }
}

#[cfg(feature = "gemini")]
fn run_with_gemini(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        GeminiModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
        cache,
    )
}

#[cfg(not(feature = "gemini"))]
fn run_with_gemini(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
    _cache: Option<&mut RunCache>,
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
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        GroqModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
        cache,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_groq(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
    _cache: Option<&mut RunCache>,
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
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        HuggingFaceModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
        cache,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_hugging_face(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
    _cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Hugging Face provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn run_with_nvidia(
    workflow: &vogon_core::Workflow,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        NvidiaModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
        cache,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_nvidia(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
    _cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "NVIDIA provider support is not enabled in this build",
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
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    run_with_adapter(
        OpenRouterModel::from_env_with_timeout_and_retries(
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
        workflow,
        redactions,
        cache,
    )
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_openrouter(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
    _cache: Option<&mut RunCache>,
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
    config: OpenAiCompatibleConfig<'_>,
    cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    let model = if config.no_auth {
        OpenAiCompatibleModel::without_authentication_with_base_url_model_timeout_and_retries(
            config.base_url,
            config.model,
            Duration::from_secs(config.timeout_seconds),
            config.max_retries,
        )?
    } else {
        OpenAiCompatibleModel::from_env_with_base_url_model_timeout_and_retries(
            config.base_url,
            config.model,
            Duration::from_secs(config.timeout_seconds),
            config.max_retries,
        )?
    };

    run_with_adapter(model, workflow, redactions, cache)
}

#[cfg(not(feature = "openai-compatible"))]
fn run_with_openai_compatible(
    _workflow: &vogon_core::Workflow,
    _redactions: &RedactionSet,
    config: OpenAiCompatibleConfig<'_>,
    _cache: Option<&mut RunCache>,
) -> Result<RunReport, Box<dyn std::error::Error>> {
    let _ = (
        config.base_url,
        config.model,
        config.no_auth,
        config.timeout_seconds,
        config.max_retries,
    );
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "OpenAI-compatible provider support is not enabled in this build",
    )
    .into())
}

fn load_run_cache(
    cache_file: Option<&Path>,
    cache_max_entries: usize,
) -> Result<Option<RunCache>, Box<dyn std::error::Error>> {
    let Some(cache_file) = cache_file else {
        return Ok(None);
    };

    if !cache_file.exists() {
        return Ok(Some(RunCache::with_max_entries(cache_max_entries)));
    }

    let cache_text = file_io::read_to_string(cache_file, "run cache file")?;
    let mut cache: RunCache = serde_json::from_str(&cache_text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse run cache file `{}`: {error}",
                cache_file.display()
            ),
        )
    })?;
    cache.set_max_entries(cache_max_entries);
    Ok(Some(cache))
}

fn write_run_cache_file(
    cache_file: &Path,
    cache: &RunCache,
) -> Result<(), Box<dyn std::error::Error>> {
    create_cache_parent(cache_file)?;
    if cache_file.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("run cache path `{}` is a directory", cache_file.display()),
        )
        .into());
    }

    let cache_json = format!("{}\n", serde_json::to_string_pretty(cache)?);
    write_replay_file(cache_file, &cache_json).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to write run cache file `{}`: {error}",
                cache_file.display()
            ),
        )
    })?;
    Ok(())
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
    create_parent(output, "replay output directory")
}

fn create_cache_parent(cache_file: &Path) -> io::Result<()> {
    create_parent(cache_file, "run cache directory")
}

fn reject_overlapping_artifact_paths(
    output: Option<&Path>,
    cache_file: Option<&Path>,
) -> io::Result<()> {
    let (Some(output), Some(cache_file)) = (output, cache_file) else {
        return Ok(());
    };

    if comparable_path(output)? == comparable_path(cache_file)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "replay output path `{}` and run cache path `{}` must be different",
                output.display(),
                cache_file.display()
            ),
        ));
    }

    Ok(())
}

fn comparable_path(path: &Path) -> io::Result<PathBuf> {
    let mut normalized = if path.is_absolute() {
        PathBuf::new()
    } else {
        env::current_dir()?
    };

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    Ok(normalized)
}

fn create_parent(path: &Path, description: &str) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "failed to create {description} `{}`: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    Ok(())
}

fn write_replay_file(output: &Path, replay_json: &str) -> io::Result<()> {
    if replay_json.len() > file_io::MAX_INPUT_FILE_BYTES as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "artifact `{}` is {} bytes, exceeding the 1 MiB limit",
                output.display(),
                replay_json.len()
            ),
        ));
    }

    let (temp_output, mut temp_file) = create_temp_output(output)?;
    let write_result = temp_file
        .write_all(replay_json.as_bytes())
        .and_then(|()| temp_file.sync_all());
    drop(temp_file);

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp_output);
        return Err(error);
    }

    let rename_result = match fs::rename(&temp_output, output) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(output).and_then(|()| fs::rename(&temp_output, output))
        }
        Err(error) => Err(error),
    };

    if rename_result.is_err() {
        let _ = fs::remove_file(&temp_output);
    }

    rename_result
}

fn create_temp_output(output: &Path) -> io::Result<(PathBuf, fs::File)> {
    for attempt in 0..100 {
        let temp_output = temp_output_path(output, attempt)?;
        match open_private_temp_file(&temp_output) {
            Ok(file) => return Ok((temp_output, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "failed to reserve a temporary file for `{}` after 100 attempts",
            output.display()
        ),
    ))
}

fn open_private_temp_file(path: &Path) -> io::Result<fs::File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    options.open(path)
}

fn temp_output_path(output: &Path, attempt: u32) -> io::Result<PathBuf> {
    let file_name = output.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("output path `{}` has no file name", output.display()),
        )
    })?;
    let temp_file_name = format!(
        ".{}.{}.{}.tmp",
        file_name.to_string_lossy(),
        process::id(),
        attempt
    );

    Ok(output.with_file_name(temp_file_name))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn artifact_write_preserves_stale_temp_file_and_uses_another_candidate() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow Unix epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!("vogon-run-artifact-{}-{unique}", process::id()));
        fs::create_dir(&root).expect("test directory should be created");
        let output = root.join("replay.json");
        let stale_temp = temp_output_path(&output, 0).expect("temporary path should be valid");
        fs::write(&stale_temp, "stale run").expect("stale temporary file should be written");

        write_replay_file(&output, "new replay\n").expect("artifact write should succeed");

        assert_eq!(fs::read_to_string(&output).unwrap(), "new replay\n");
        assert_eq!(fs::read_to_string(&stale_temp).unwrap(), "stale run");
        assert!(!temp_output_path(&output, 1).unwrap().exists());
        fs::remove_dir_all(root).expect("test directory should be removed");
    }

    #[test]
    fn artifact_write_rejects_files_that_cannot_be_reopened() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow Unix epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!(
            "vogon-run-oversized-artifact-{}-{unique}",
            process::id()
        ));
        fs::create_dir(&root).expect("test directory should be created");
        let output = root.join("replay.json");
        fs::write(&output, "existing replay\n").expect("existing replay should be written");
        let oversized = "x".repeat(file_io::MAX_INPUT_FILE_BYTES as usize + 1);

        let error = write_replay_file(&output, &oversized).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("exceeding the 1 MiB limit"));
        assert_eq!(
            fs::read_to_string(&output).unwrap(),
            "existing replay\n",
            "an invalid artifact must not replace the existing replay"
        );
        assert!(!temp_output_path(&output, 0).unwrap().exists());
        fs::remove_dir_all(root).expect("test directory should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn artifact_write_uses_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow Unix epoch")
            .as_nanos();
        let root = env::temp_dir().join(format!(
            "vogon-run-private-artifact-{}-{unique}",
            process::id()
        ));
        fs::create_dir(&root).expect("test directory should be created");
        let output = root.join("replay.json");

        write_replay_file(&output, "private replay\n").expect("artifact write should succeed");

        let mode = fs::metadata(&output).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        fs::remove_dir_all(root).expect("test directory should be removed");
    }
}
