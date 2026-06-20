#![forbid(unsafe_code)]

mod commands;

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

use commands::run::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_TIMEOUT_SECONDS, DEFAULT_OPENAI_COMPATIBLE_BASE_URL,
    DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, DEFAULT_OPENAI_COMPATIBLE_MODEL,
    DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, MAX_GEMINI_RETRIES, MAX_OPENAI_COMPATIBLE_RETRIES,
    ModelProvider, RunModelConfig,
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

        /// OpenAI-compatible base URL when `--provider openai-compatible` is selected.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_BASE_URL)]
        openai_compatible_base_url: String,

        /// OpenAI-compatible model name when `--provider openai-compatible` is selected.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_MODEL)]
        openai_compatible_model: String,

        /// OpenAI-compatible request timeout in seconds.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openai_compatible_timeout_seconds: u64,

        /// OpenAI-compatible retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, value_parser = parse_openai_compatible_max_retries)]
        openai_compatible_max_retries: u32,

        /// Redact a literal value from replay outputs. May be repeated.
        #[arg(long = "redact", value_name = "LABEL=VALUE")]
        redactions: Vec<String>,

        /// Write the replay JSON to a file instead of stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

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

        /// OpenAI-compatible base URL when verifying with an OpenAI-compatible provider.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_BASE_URL)]
        openai_compatible_base_url: String,

        /// OpenAI-compatible model name when verifying with an OpenAI-compatible provider.
        #[arg(long, default_value = DEFAULT_OPENAI_COMPATIBLE_MODEL)]
        openai_compatible_model: String,

        /// OpenAI-compatible request timeout in seconds.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, value_parser = clap::value_parser!(u64).range(1..))]
        openai_compatible_timeout_seconds: u64,

        /// OpenAI-compatible retry count for transient provider errors.
        #[arg(long, default_value_t = DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES, value_parser = parse_openai_compatible_max_retries)]
        openai_compatible_max_retries: u32,

        /// Redact a literal value before comparing replay outputs. May be repeated.
        #[arg(long = "redact", value_name = "LABEL=VALUE")]
        redactions: Vec<String>,

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

        /// Emit newline-delimited JSON instead of human-readable text.
        #[arg(long)]
        jsonl: bool,

        replay_file: PathBuf,
    },
}

fn parse_gemini_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(value, "--gemini-max-retries", MAX_GEMINI_RETRIES)
}

fn parse_openai_compatible_max_retries(value: &str) -> Result<u32, String> {
    parse_retry_count(
        value,
        "--openai-compatible-max-retries",
        MAX_OPENAI_COMPATIBLE_RETRIES,
    )
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
        Commands::Providers { json } => commands::providers::run(json),
        Commands::Run {
            provider,
            gemini_model,
            gemini_timeout_seconds,
            gemini_max_retries,
            openai_compatible_base_url,
            openai_compatible_model,
            openai_compatible_timeout_seconds,
            openai_compatible_max_retries,
            output,
            redactions,
            workflow_file,
        } => commands::run::run(
            &workflow_file,
            &redactions,
            output.as_deref(),
            RunModelConfig {
                provider,
                gemini_model: &gemini_model,
                gemini_timeout_seconds,
                gemini_max_retries,
                openai_compatible_base_url: &openai_compatible_base_url,
                openai_compatible_model: &openai_compatible_model,
                openai_compatible_timeout_seconds,
                openai_compatible_max_retries,
            },
        ),
        Commands::Verify {
            provider,
            gemini_model,
            gemini_timeout_seconds,
            gemini_max_retries,
            openai_compatible_base_url,
            openai_compatible_model,
            openai_compatible_timeout_seconds,
            openai_compatible_max_retries,
            redactions,
            json,
            workflow_file,
            replay_file,
        } => commands::verify::run(
            &workflow_file,
            &replay_file,
            &redactions,
            json,
            VerifyModelConfig {
                provider,
                gemini_model: &gemini_model,
                gemini_timeout_seconds,
                gemini_max_retries,
                openai_compatible_base_url: &openai_compatible_base_url,
                openai_compatible_model: &openai_compatible_model,
                openai_compatible_timeout_seconds,
                openai_compatible_max_retries,
            },
        ),
        Commands::Trace {
            redactions,
            jsonl,
            replay_file,
        } => commands::trace::run(&replay_file, jsonl, &redactions),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
