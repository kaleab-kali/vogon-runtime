use serde::Serialize;
use vogon_adapters::DeterministicEchoModel;
use vogon_core::{Runtime, Step, StepId, Workflow};

use crate::commands::providers::{ProviderStatus, print_provider_human, provider_statuses};

pub fn run(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let report = DoctorReport::new()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    status: &'static str,
    version: &'static str,
    checks: Vec<DoctorCheck>,
    providers: Vec<ProviderStatus>,
}

impl DoctorReport {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let checks = vec![deterministic_runtime_check()?];

        Ok(Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            checks,
            providers: provider_statuses(),
        })
    }
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: &'static str,
    status: &'static str,
    message: String,
}

fn deterministic_runtime_check() -> Result<DoctorCheck, Box<dyn std::error::Error>> {
    let workflow = Workflow::new(
        "doctor",
        vec![Step::new(
            StepId::new("self_check")?,
            "Run a deterministic runtime self-check.",
        )],
    )?;
    let report = Runtime::new(DeterministicEchoModel).run(&workflow)?;

    if report.steps.len() != 1 {
        return Err(format!(
            "deterministic runtime self-check produced {} steps, expected 1",
            report.steps.len()
        )
        .into());
    }

    Ok(DoctorCheck {
        name: "deterministic_runtime",
        status: "ok",
        message: "deterministic runtime executed a one-step workflow".to_owned(),
    })
}

fn print_human(report: &DoctorReport) {
    println!("Doctor status: {}", report.status);
    println!("Version: {}", report.version);
    println!("Checks:");
    for check in &report.checks {
        println!("- {}: {} ({})", check.name, check.status, check.message);
    }
    println!("Providers:");
    for provider in &report.providers {
        print_provider_human(provider);
    }
}
