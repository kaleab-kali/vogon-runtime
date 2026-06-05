#![forbid(unsafe_code)]

mod commands;

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

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

    /// Run a workflow file.
    Run {
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Check {
            json,
            workflow_file,
        } => commands::check::run(&workflow_file, json),
        Commands::Demo => commands::demo::run(),
        Commands::Run {
            output,
            redactions,
            workflow_file,
        } => commands::run::run(&workflow_file, &redactions, output.as_deref()),
        Commands::Verify {
            redactions,
            json,
            workflow_file,
            replay_file,
        } => commands::verify::run(&workflow_file, &replay_file, &redactions, json),
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
