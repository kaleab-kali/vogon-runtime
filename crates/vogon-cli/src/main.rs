mod commands;

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "vogon")]
#[command(about = "Deterministic, replayable AI workflow runtime.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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

        workflow_file: PathBuf,
        replay_file: PathBuf,
    },

    /// Print a trace for a replay file.
    Trace {
        /// Emit newline-delimited JSON instead of human-readable text.
        #[arg(long)]
        jsonl: bool,

        replay_file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Demo => commands::demo::run(),
        Commands::Run {
            output,
            redactions,
            workflow_file,
        } => commands::run::run(&workflow_file, &redactions, output.as_deref()),
        Commands::Verify {
            redactions,
            workflow_file,
            replay_file,
        } => commands::verify::run(&workflow_file, &replay_file, &redactions),
        Commands::Trace { jsonl, replay_file } => commands::trace::run(&replay_file, jsonl),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
