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
    Run { workflow_file: PathBuf },

    /// Verify a workflow against a replay file.
    Verify {
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
        Commands::Run { workflow_file } => commands::run::run(&workflow_file),
        Commands::Verify {
            workflow_file,
            replay_file,
        } => commands::verify::run(&workflow_file, &replay_file),
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
