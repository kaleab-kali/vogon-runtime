#[cfg(any(feature = "gemini", feature = "openai-compatible"))]
use std::time::Duration;
use std::{collections::BTreeSet, io, path::Path};

use serde_json::json;
use vogon_adapters::DeterministicEchoModel;
#[cfg(feature = "gemini")]
use vogon_adapters::GeminiModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::GroqModel;
#[cfg(feature = "openai-compatible")]
use vogon_adapters::OpenAiCompatibleModel;
use vogon_core::{RedactionSet, ReplayMismatch, RunReport, Runtime, VerificationReport};

use crate::commands::file_io;
use crate::commands::redaction::parse_redactions;
use crate::commands::redaction_markers::replay_redaction_labels;
use crate::commands::run::{
    DEFAULT_GEMINI_MAX_RETRIES, DEFAULT_GEMINI_TIMEOUT_SECONDS, DEFAULT_GROQ_MAX_RETRIES,
    DEFAULT_GROQ_TIMEOUT_SECONDS, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES,
    DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS, ModelProvider,
};
use crate::commands::workflow_file::read_toml_workflow;

const REDACTED_MISMATCH_OUTPUT: &str = "[UNREPORTED: replay is redacted]";

pub fn run(
    workflow_file: &Path,
    replay_file: &Path,
    redaction_values: &[String],
    json: bool,
    model_config: VerifyModelConfig<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = read_toml_workflow(workflow_file)?;
    let replay_text = file_io::read_to_string(replay_file, "replay file")?;
    let replay: RunReport = serde_json::from_str(&replay_text).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "failed to parse replay file `{}`: {error}",
                replay_file.display()
            ),
        )
    })?;
    let redactions = parse_redactions(redaction_values)?;
    let replay_redaction_labels = replay_redaction_labels(&replay)?;
    let missing_redaction_labels =
        missing_replay_redaction_labels(&replay_redaction_labels, &redactions);
    if !missing_redaction_labels.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "replay contains redaction marker(s) without matching --redact label(s): {}",
                missing_redaction_labels.join(", ")
            ),
        )
        .into());
    }

    let resolved_model_config = resolve_model_config(&replay, model_config)?;
    let verification = verify_with_model(&workflow, &replay, &redactions, resolved_model_config)?;

    let mismatch_count = verification.mismatches.len();
    let printable_verification = redact_step_output_mismatches(verification, &redactions);
    let printable_verification = if replay_redaction_labels.is_empty() {
        printable_verification
    } else {
        mask_redacted_step_outputs(printable_verification)
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_verification_report(&printable_verification))?
        );

        if printable_verification.is_match() {
            return Ok(());
        }

        return Err(io::Error::other(format!(
            "replay verification failed with {mismatch_count} mismatch(es)"
        ))
        .into());
    }

    if printable_verification.is_match() {
        println!(
            "Replay verified: {} ({} steps)",
            replay.workflow_name,
            replay.steps.len()
        );
        return Ok(());
    }

    eprintln!("{}", serde_json::to_string_pretty(&printable_verification)?);
    Err(io::Error::other(format!(
        "replay verification failed with {mismatch_count} mismatch(es)"
    ))
    .into())
}

#[derive(Debug, Clone, Copy)]
pub struct VerifyModelConfig<'a> {
    pub provider: Option<ModelProvider>,
    pub gemini_model: &'a str,
    pub gemini_timeout_seconds: u64,
    pub gemini_max_retries: u32,
    pub groq_model: &'a str,
    pub groq_timeout_seconds: u64,
    pub groq_max_retries: u32,
    pub openai_compatible_base_url: &'a str,
    pub openai_compatible_model: &'a str,
    pub openai_compatible_timeout_seconds: u64,
    pub openai_compatible_max_retries: u32,
}

#[derive(Debug, Clone)]
struct ResolvedModelConfig {
    provider: ModelProvider,
    gemini_model: String,
    gemini_timeout_seconds: u64,
    gemini_max_retries: u32,
    groq_model: String,
    groq_timeout_seconds: u64,
    groq_max_retries: u32,
    openai_compatible_base_url: String,
    openai_compatible_model: String,
    openai_compatible_timeout_seconds: u64,
    openai_compatible_max_retries: u32,
}

fn resolve_model_config(
    replay: &RunReport,
    model_config: VerifyModelConfig<'_>,
) -> Result<ResolvedModelConfig, Box<dyn std::error::Error>> {
    let provider = match model_config.provider {
        Some(provider) => provider,
        None => ModelProvider::from_runtime_provider_name(&replay.runtime.provider).ok_or_else(
            || {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "replay provider `{}` is not supported by this CLI",
                        replay.runtime.provider
                    ),
                )
            },
        )?,
    };

    let use_replay_metadata = model_config.provider.is_none();

    Ok(ResolvedModelConfig {
        provider,
        gemini_model: if use_replay_metadata && provider == ModelProvider::Gemini {
            replay
                .runtime
                .model
                .clone()
                .unwrap_or_else(|| model_config.gemini_model.to_owned())
        } else {
            model_config.gemini_model.to_owned()
        },
        gemini_timeout_seconds: if use_replay_metadata && provider == ModelProvider::Gemini {
            replay_timeout_seconds(replay, DEFAULT_GEMINI_TIMEOUT_SECONDS)?
        } else {
            model_config.gemini_timeout_seconds
        },
        gemini_max_retries: if use_replay_metadata && provider == ModelProvider::Gemini {
            replay_max_retries(replay, DEFAULT_GEMINI_MAX_RETRIES)?
        } else {
            model_config.gemini_max_retries
        },
        groq_model: if use_replay_metadata && provider == ModelProvider::Groq {
            replay
                .runtime
                .model
                .clone()
                .unwrap_or_else(|| model_config.groq_model.to_owned())
        } else {
            model_config.groq_model.to_owned()
        },
        groq_timeout_seconds: if use_replay_metadata && provider == ModelProvider::Groq {
            replay_timeout_seconds(replay, DEFAULT_GROQ_TIMEOUT_SECONDS)?
        } else {
            model_config.groq_timeout_seconds
        },
        groq_max_retries: if use_replay_metadata && provider == ModelProvider::Groq {
            replay_max_retries(replay, DEFAULT_GROQ_MAX_RETRIES)?
        } else {
            model_config.groq_max_retries
        },
        openai_compatible_base_url: if use_replay_metadata
            && provider == ModelProvider::OpenAiCompatible
        {
            replay_parameter(replay, "base_url")
                .unwrap_or(model_config.openai_compatible_base_url)
                .to_owned()
        } else {
            model_config.openai_compatible_base_url.to_owned()
        },
        openai_compatible_model: if use_replay_metadata
            && provider == ModelProvider::OpenAiCompatible
        {
            replay
                .runtime
                .model
                .clone()
                .unwrap_or_else(|| model_config.openai_compatible_model.to_owned())
        } else {
            model_config.openai_compatible_model.to_owned()
        },
        openai_compatible_timeout_seconds: if use_replay_metadata
            && provider == ModelProvider::OpenAiCompatible
        {
            replay_timeout_seconds(replay, DEFAULT_OPENAI_COMPATIBLE_TIMEOUT_SECONDS)?
        } else {
            model_config.openai_compatible_timeout_seconds
        },
        openai_compatible_max_retries: if use_replay_metadata
            && provider == ModelProvider::OpenAiCompatible
        {
            replay_max_retries(replay, DEFAULT_OPENAI_COMPATIBLE_MAX_RETRIES)?
        } else {
            model_config.openai_compatible_max_retries
        },
    })
}

fn replay_parameter<'a>(replay: &'a RunReport, key: &str) -> Option<&'a str> {
    replay.runtime.parameters.get(key).map(String::as_str)
}

fn replay_timeout_seconds(
    replay: &RunReport,
    default_seconds: u64,
) -> Result<u64, Box<dyn std::error::Error>> {
    let Some(timeout_nanos) = replay_parameter(replay, "timeout_nanos") else {
        return Ok(default_seconds);
    };
    let timeout_nanos = timeout_nanos.parse::<u128>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("replay timeout_nanos `{timeout_nanos}` is invalid: {error}"),
        )
    })?;
    const NANOS_PER_SECOND: u128 = 1_000_000_000;

    if timeout_nanos == 0 || timeout_nanos % NANOS_PER_SECOND != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "replay timeout_nanos `{timeout_nanos}` cannot be represented by CLI timeout seconds"
            ),
        )
        .into());
    }

    let timeout_seconds = timeout_nanos / NANOS_PER_SECOND;
    u64::try_from(timeout_seconds).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("replay timeout_nanos `{timeout_nanos}` is too large: {error}"),
        )
        .into()
    })
}

fn replay_max_retries(
    replay: &RunReport,
    default_max_retries: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    let Some(max_retries) = replay_parameter(replay, "max_retries") else {
        return Ok(default_max_retries);
    };

    max_retries.parse::<u32>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("replay max_retries `{max_retries}` is invalid: {error}"),
        )
        .into()
    })
}

fn verify_with_model(
    workflow: &vogon_core::Workflow,
    replay: &RunReport,
    redactions: &RedactionSet,
    model_config: ResolvedModelConfig,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    match model_config.provider {
        ModelProvider::Deterministic => Ok(Runtime::new(DeterministicEchoModel)
            .verify_with_redactions(workflow, replay, redactions)?),
        ModelProvider::Gemini => verify_with_gemini(
            workflow,
            replay,
            redactions,
            &model_config.gemini_model,
            model_config.gemini_timeout_seconds,
            model_config.gemini_max_retries,
        ),
        ModelProvider::Groq => verify_with_groq(
            workflow,
            replay,
            redactions,
            &model_config.groq_model,
            model_config.groq_timeout_seconds,
            model_config.groq_max_retries,
        ),
        ModelProvider::OpenAiCompatible => verify_with_openai_compatible(
            workflow,
            replay,
            redactions,
            &model_config.openai_compatible_base_url,
            &model_config.openai_compatible_model,
            model_config.openai_compatible_timeout_seconds,
            model_config.openai_compatible_max_retries,
        ),
    }
}

#[cfg(feature = "gemini")]
fn verify_with_gemini(
    workflow: &vogon_core::Workflow,
    replay: &RunReport,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Ok(Runtime::new(GeminiModel::from_env_with_timeout_and_retries(
        model,
        Duration::from_secs(timeout_seconds),
        max_retries,
    )?)
    .verify_with_redactions(workflow, replay, redactions)?)
}

#[cfg(not(feature = "gemini"))]
fn verify_with_gemini(
    _workflow: &vogon_core::Workflow,
    _replay: &RunReport,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Gemini provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn verify_with_groq(
    workflow: &vogon_core::Workflow,
    replay: &RunReport,
    redactions: &RedactionSet,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Ok(Runtime::new(GroqModel::from_env_with_timeout_and_retries(
        model,
        Duration::from_secs(timeout_seconds),
        max_retries,
    )?)
    .verify_with_redactions(workflow, replay, redactions)?)
}

#[cfg(not(feature = "openai-compatible"))]
fn verify_with_groq(
    _workflow: &vogon_core::Workflow,
    _replay: &RunReport,
    _redactions: &RedactionSet,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Groq provider support is not enabled in this build",
    )
    .into())
}

#[cfg(feature = "openai-compatible")]
fn verify_with_openai_compatible(
    workflow: &vogon_core::Workflow,
    replay: &RunReport,
    redactions: &RedactionSet,
    base_url: &str,
    model: &str,
    timeout_seconds: u64,
    max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Ok(Runtime::new(
        OpenAiCompatibleModel::from_env_with_base_url_model_timeout_and_retries(
            base_url,
            model,
            Duration::from_secs(timeout_seconds),
            max_retries,
        )?,
    )
    .verify_with_redactions(workflow, replay, redactions)?)
}

#[cfg(not(feature = "openai-compatible"))]
fn verify_with_openai_compatible(
    _workflow: &vogon_core::Workflow,
    _replay: &RunReport,
    _redactions: &RedactionSet,
    _base_url: &str,
    _model: &str,
    _timeout_seconds: u64,
    _max_retries: u32,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "OpenAI-compatible provider support is not enabled in this build",
    )
    .into())
}

fn json_verification_report(report: &VerificationReport) -> serde_json::Value {
    json!({
        "workflow_name": report.workflow_name,
        "is_match": report.is_match(),
        "mismatches": report.mismatches,
    })
}

fn missing_replay_redaction_labels(
    replay_labels: &BTreeSet<String>,
    redactions: &RedactionSet,
) -> Vec<String> {
    let configured_labels = redactions
        .rules()
        .iter()
        .map(|rule| rule.label.as_str())
        .collect::<BTreeSet<_>>();

    replay_labels
        .iter()
        .filter(|label| !configured_labels.contains(label.as_str()))
        .cloned()
        .collect()
}

fn mask_redacted_step_outputs(report: VerificationReport) -> VerificationReport {
    VerificationReport {
        workflow_name: report.workflow_name,
        mismatches: report
            .mismatches
            .into_iter()
            .map(mask_redacted_step_output)
            .collect(),
    }
}

fn mask_redacted_step_output(mismatch: ReplayMismatch) -> ReplayMismatch {
    match mismatch {
        ReplayMismatch::StepOutput {
            step_id,
            expected: _,
            actual: _,
        } => ReplayMismatch::StepOutput {
            step_id,
            expected: REDACTED_MISMATCH_OUTPUT.to_owned(),
            actual: REDACTED_MISMATCH_OUTPUT.to_owned(),
        },
        other => other,
    }
}

fn redact_step_output_mismatches(
    report: VerificationReport,
    redactions: &RedactionSet,
) -> VerificationReport {
    VerificationReport {
        workflow_name: report.workflow_name,
        mismatches: report
            .mismatches
            .into_iter()
            .map(|mismatch| redact_step_output_mismatch(mismatch, redactions))
            .collect(),
    }
}

fn redact_step_output_mismatch(
    mismatch: ReplayMismatch,
    redactions: &RedactionSet,
) -> ReplayMismatch {
    match mismatch {
        ReplayMismatch::StepOutput {
            step_id,
            expected,
            actual,
        } => ReplayMismatch::StepOutput {
            step_id,
            expected: redactions.redact(&expected),
            actual: redactions.redact(&actual),
        },
        other => other,
    }
}
