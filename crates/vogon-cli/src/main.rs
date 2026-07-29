#![forbid(unsafe_code)]

mod commands;

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use vogon_core::DEFAULT_RUN_CACHE_MAX_ENTRIES;

use commands::run::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_TIMEOUT_SECONDS, DEFAULT_GROQ_MAX_RETRIES,
    DEFAULT_GROQ_MODEL, DEFAULT_GROQ_TIMEOUT_SECONDS, DEFAULT_HUGGING_FACE_MAX_RETRIES,
    DEFAULT_HUGGING_FACE_MODEL, DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS, DEFAULT_NVIDIA_MAX_RETRIES,
    DEFAULT_NVIDIA_MODEL, DEFAULT_NVIDIA_TIMEOUT_SECONDS, DEFAULT_OPENAI_COMPATIBLE_BASE_URL,
    DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, DEFAULT_OPENAI_COMPATIBLE_MODEL,
    DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, DEFAULT_OPENROUTER_MAX_RETRIES,
    DEFAULT_OPENROUTER_MODEL, DEFAULT_OPENROUTER_TIMEOUT_SECONDS, MAX_GEMINI_RETRIES,
    MAX_GROQ_RETRIES, MAX_HUGGING_FACE_RETRIES, MAX_NVIDIA_RETRIES, MAX_OPENAI_COMPATIBLE_RETRIES,
    MAX_OPENROUTER_RETRIES, ModelProvider, RunModelConfig,
};
use commands::verify::VerifyModelConfig;

#[derive(Debug, Parser)]
#[command(name = "vogon")]
#[command(about = "Deterministic, replayable AI workflow runtime.")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate a workflow file without executing it.
    Check {
        /// Emit machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,

        workflow_file: PathBuf,
    },

    /// Run the built-in deterministic demo workflow.
    Demo,

    /// Run local installation diagnostics without making network calls.
    Doctor {
        /// Emit machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },

    /// Create a starter TOML workflow file.
    Init {
        /// Output path for the generated workflow file.
        #[arg(short, long, default_value = "workflow.toml", value_name = "FILE")]
        output: PathBuf,

        /// Overwrite an existing workflow file.
        #[arg(long)]
        force: bool,
    },

    /// Show available model providers and credential configuration status.
    Providers {
        /// Emit machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },

    /// Run a workflow file.
    Run {
        /// Model provider to use for workflow execution.
        #[arg(long, value_enum, default_value_t = ModelProvider::Deterministic)]
        provider: ModelProvider,

        /// Gemini model name when `--provider gemini` is selected.
        #[arg(long, default_value = "gemini-3.1-flash-lite")]
        gemini_model: String,

        /// Gemini request timeout in seconds when `--provider gemini` is selected.
        #[arg(long, default_value_t = DEFAULT_GEMINI_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        gemini_timeout_seconds: u64,

        /// Gemini retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_GEMINI_MAX_RETRIES, value_parser = parse_gemini_max_retries)]
        gemini_max_retries: u32,

        /// Groq model name when `--provider groq` is selected.
        #[arg(long, default_value = DEFAULT_GROQ_MODEL)]
        groq_model: String,

        /// Groq request timeout in seconds when `--provider groq` is selected.
        #[arg(long, default_value_t = DEFAULT_GROQ_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        groq_timeout_seconds: u64,

        /// Groq retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_GROQ_MAX_RETRIES, value_parser = parse_groq_max_retries)]
        groq_max_retries: u32,

        /// Hugging Face model name when `--provider hugging-face` is selected.
        #[arg(long, default_value = DEFAULT_HUGGING_FACE_MODEL)]
        hugging_face_model: String,

        /// Hugging Face request timeout in seconds when `--provider hugging-face` is selected.
        #[arg(long, default_value_t = DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        hugging_face_timeout_seconds: u64,

        /// Hugging Face retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_HUGGING_FACE_MAX_RETRIES, value_parser = parse_hugging_face_max_retries)]
        hugging_face_max_retries: u32,

        /// NVIDIA model name when `--provider nvidia` is selected.
        #[arg(long, default_value = DEFAULT_NVIDIA_MODEL)]
        nvidia_model: String,

        /// NVIDIA request timeout in seconds when `--provider nvidia` is selected.
        #[arg(long, default_value_t = DEFAULT_NVIDIA_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        nvidia_timeout_seconds: u64,

        /// NVIDIA retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_NVIDIA_MAX_RETRIES, value_parser = parse_nvidia_max_retries)]
        nvidia_max_retries: u32,

        /// OpenRouter model name when `--provider openrouter` is selected.
        #[arg(long, default_value = DEFAULT_OPENROUTER_MODEL)]
        openrouter_model: String,

        /// OpenRouter request timeout in seconds when `--provider openrouter` is selected.
        #[arg(long, default_value_t = DEFAULT_OPENROUTER_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openrouter_timeout_seconds: u64,

        /// OpenRouter retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENROUTER_MAX_RETRIES, value_parser = parse_openrouter_max_retries)]
        openrouter_max_retries: u32,

        /// OpenAI-compatible base URL when `--provider openai-compatible` is selected.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_BASE_URL)]
        openai_compatible_base_url: String,

        /// OpenAI-compatible model name when `--provider openai-compatible` is selected.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_MODEL)]
        openai_compatible_model: String,

        /// Omit authentication for an explicitly selected OpenAI-compatible endpoint.
        #[arg(long)]
        openai_compatible_no_auth: bool,

        /// OpenAI-compatible request timeout in seconds.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openai_compatible_timeout_seconds: u64,

        /// OpenAI-compatible retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, value_parser = parse_openai_compatible_max_retries)]
        openai_compatible_max_retries: u32,

        /// Redact a literal value from replay outputs. May be repeated.
        #[arg(long = "redact", value_name = "LABEL=VALUE")]
        redactions: Vec<String>,

        /// Redact a value read from an environment variable. May be repeated.
        #[arg(long = "redact-env", value_name = "LABEL=ENV_VAR")]
        redaction_environment_values: Vec<String>,

        /// Write the replay JSON to a file instead of stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Persist provider outputs in a bounded cache file for repeated runs.
        #[arg(long, value_name = "FILE")]
        cache_file: Option<PathBuf>,

        /// Maximum number of outputs retained in `--cache-file`.
        #[arg(long, default_value_t = DEFAULT_RUN_CACHE_MAX_ENTRIES)]
        cache_max_entries: usize,

        workflow_file: PathBuf,
    },

    /// Verify a workflow against a replay file.
    Verify {
        /// Model provider to use for verification. Defaults to the replay provider.
        #[arg(long, value_enum)]
        provider: Option<ModelProvider>,

        /// Gemini model name when verifying with the Gemini provider.
        #[arg(long, default_value = "gemini-3.1-flash-lite")]
        gemini_model: String,

        /// Gemini request timeout in seconds when verifying with Gemini.
        #[arg(long, default_value_t = DEFAULT_GEMINI_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        gemini_timeout_seconds: u64,

        /// Gemini retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_GEMINI_MAX_RETRIES, value_parser = parse_gemini_max_retries)]
        gemini_max_retries: u32,

        /// Groq model name when verifying with the Groq provider.
        #[arg(long, default_value = DEFAULT_GROQ_MODEL)]
        groq_model: String,

        /// Groq request timeout in seconds when verifying with Groq.
        #[arg(long, default_value_t = DEFAULT_GROQ_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        groq_timeout_seconds: u64,

        /// Groq retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_GROQ_MAX_RETRIES, value_parser = parse_groq_max_retries)]
        groq_max_retries: u32,

        /// Hugging Face model name when verifying with the Hugging Face provider.
        #[arg(long, default_value = DEFAULT_HUGGING_FACE_MODEL)]
        hugging_face_model: String,

        /// Hugging Face request timeout in seconds when verifying with Hugging Face.
        #[arg(long, default_value_t = DEFAULT_HUGGING_FACE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        hugging_face_timeout_seconds: u64,

        /// Hugging Face retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_HUGGING_FACE_MAX_RETRIES, value_parser = parse_hugging_face_max_retries)]
        hugging_face_max_retries: u32,

        /// NVIDIA model name when verifying with the NVIDIA provider.
        #[arg(long, default_value = DEFAULT_NVIDIA_MODEL)]
        nvidia_model: String,

        /// NVIDIA request timeout in seconds when verifying with NVIDIA.
        #[arg(long, default_value_t = DEFAULT_NVIDIA_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        nvidia_timeout_seconds: u64,

        /// NVIDIA retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_NVIDIA_MAX_RETRIES, value_parser = parse_nvidia_max_retries)]
        nvidia_max_retries: u32,

        /// OpenRouter model name when verifying with the OpenRouter provider.
        #[arg(long, default_value = DEFAULT_OPENROUTER_MODEL)]
        openrouter_model: String,

        /// OpenRouter request timeout in seconds when verifying with OpenRouter.
        #[arg(long, default_value_t = DEFAULT_OPENROUTER_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openrouter_timeout_seconds: u64,

        /// OpenRouter retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENROUTER_MAX_RETRIES, value_parser = parse_openrouter_max_retries)]
        openrouter_max_retries: u32,

        /// OpenAI-compatible base URL when verifying with an OpenAI-compatible provider.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_BASE_URL)]
        openai_compatible_base_url: String,

        /// OpenAI-compatible model name when verifying with an OpenAI-compatible provider.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_MODEL)]
        openai_compatible_model: String,

        /// Omit authentication for an explicitly selected OpenAI-compatible endpoint.
        #[arg(long)]
        openai_compatible_no_auth: bool,

        /// OpenAI-compatible request timeout in seconds.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openai_compatible_timeout_seconds: u64,

        /// OpenAI-compatible retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, value_parser = parse_openai_compatible_max_retries)]
        openai_compatible_max_retries: u32,

        /// Redact a literal value before comparing replay outputs. May be repeated.
        #[arg(long = "redact", value_name = "LABEL=VALUE")]
        redactions: Vec<String>,

        /// Redact a value read from an environment variable. May be repeated.
        #[arg(long = "redact-env", value_name = "LABEL=ENV_VAR")]
        redaction_environment_values: Vec<String>,

        /// Emit machine-readable JSON instead of human-readable text.
        #[arg(long)]
        json: bool,

        workflow_file: PathBuf,
        replay_file: PathBuf,
    },

    /// Print a trace for a replay file.
    Trace {
        /// Redact a literal value from trace outputs. May be repeated.
        #[arg(long = "redact", value_name = "LABEL=VALUE")]
        redactions: Vec<String>,

        /// Redact a value read from an environment variable. May be repeated.
        #[arg(long = "redact-env", value_name = "LABEL=ENV_VAR")]
        redaction_environment_values: Vec<String>,

        /// Emit newline-delimited JSON instead of human-readable text.
        #[arg(long)]
        jsonl: bool,

        replay_file: PathBuf,
    },
}

fn parse_gemini_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(value, "--gemini-max-retries", MAX_GEMINI_RETRIES)
}

fn parse_groq_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(value, "--groq-max-retries", MAX_GROQ_RETRIES)
}

fn parse_hugging_face_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(
        value,
        "--hugging-face-max-retries",
        MAX_HUGGING_FACE_RETRIES,
    )
}

fn parse_nvidia_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(value, "--nvidia-max-retries", MAX_NVIDIA_RETRIES)
}

fn parse_openai_compatible_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(
        value,
        "--openai-compatible-max-retries",
        MAX_OPENAI_COMPATIBLE_RETRIES,
    )
}

fn parse_openrouter_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(value, "--openrouter-max-retries", MAX_OPENROUTER_RETRIES)
}

fn parse_retry_count(value: &str, option: &str, max_retries: u32) -> Result<u32, String> {
    let retries = value
        .parse::<u32>()
        .map_err(|error| format!("{option} must be between 0 and {max_retries}: {error}"))?;

    if retries > max_retries {
        return Err(format!("{option} must be between 0 and {max_retries}"));
    }

    Ok(retries)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Check {
            json,
            workflow_file,
        } => commands::check::run(&workflow_file, json),
        Commands::Demo => commands::demo::run(),
        Commands::Doctor { json } => commands::doctor::run(json),
        Commands::Init { output, force } => commands::init::run(&output, force),
        Commands::Providers { json } => commands::providers::run(json),
        Commands::Run {
            provider,
            gemini_model,
            gemini_timeout_seconds,
            gemini_max_retries,
            groq_model,
            groq_timeout_seconds,
            groq_max_retries,
            hugging_face_model,
            hugging_face_timeout_seconds,
            hugging_face_max_retries,
            nvidia_model,
            nvidia_timeout_seconds,
            nvidia_max_retries,
            openrouter_model,
            openrouter_timeout_seconds,
            openrouter_max_retries,
            openai_compatible_base_url,
            openai_compatible_model,
            openai_compatible_no_auth,
            openai_compatible_timeout_seconds,
            openai_compatible_max_retries,
            output,
            cache_file,
            cache_max_entries,
            redactions,
            redaction_environment_values,
            workflow_file,
        } => commands::run::run(
            &workflow_file,
            &redactions,
            &redaction_environment_values,
            output.as_deref(),
            cache_file.as_deref(),
            cache_max_entries,
            RunModelConfig {
                provider,
                gemini_model: &gemini_model,
                gemini_timeout_seconds,
                gemini_max_retries,
                groq_model: &groq_model,
                groq_timeout_seconds,
                groq_max_retries,
                hugging_face_model: &hugging_face_model,
                hugging_face_timeout_seconds,
                hugging_face_max_retries,
                nvidia_model: &nvidia_model,
                nvidia_timeout_seconds,
                nvidia_max_retries,
                openrouter_model: &openrouter_model,
                openrouter_timeout_seconds,
                openrouter_max_retries,
                openai_compatible_base_url: &openai_compatible_base_url,
                openai_compatible_model: &openai_compatible_model,
                openai_compatible_no_auth,
                openai_compatible_timeout_seconds,
                openai_compatible_max_retries,
            },
        ),
        Commands::Verify {
            provider,
            gemini_model,
            gemini_timeout_seconds,
            gemini_max_retries,
            groq_model,
            groq_timeout_seconds,
            groq_max_retries,
            hugging_face_model,
            hugging_face_timeout_seconds,
            hugging_face_max_retries,
            nvidia_model,
            nvidia_timeout_seconds,
            nvidia_max_retries,
            openrouter_model,
            openrouter_timeout_seconds,
            openrouter_max_retries,
            openai_compatible_base_url,
            openai_compatible_model,
            openai_compatible_no_auth,
            openai_compatible_timeout_seconds,
            openai_compatible_max_retries,
            redactions,
            redaction_environment_values,
            json,
            workflow_file,
            replay_file,
        } => commands::verify::run(
            &workflow_file,
            &replay_file,
            &redactions,
            &redaction_environment_values,
            json,
            VerifyModelConfig {
                provider,
                gemini_model: &gemini_model,
                gemini_timeout_seconds,
                gemini_max_retries,
                groq_model: &groq_model,
                groq_timeout_seconds,
                groq_max_retries,
                hugging_face_model: &hugging_face_model,
                hugging_face_timeout_seconds,
                hugging_face_max_retries,
                nvidia_model: &nvidia_model,
                nvidia_timeout_seconds,
                nvidia_max_retries,
                openrouter_model: &openrouter_model,
                openrouter_timeout_seconds,
                openrouter_max_retries,
                openai_compatible_base_url: &openai_compatible_base_url,
                openai_compatible_model: &openai_compatible_model,
                openai_compatible_no_auth,
                openai_compatible_timeout_seconds,
                openai_compatible_max_retries,
            },
        ),
        Commands::Trace {
            redactions,
            redaction_environment_values,
            jsonl,
            replay_file,
        } => commands::trace::run(
            &replay_file,
            jsonl,
            &redactions,
            &redaction_environment_values,
        ),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
