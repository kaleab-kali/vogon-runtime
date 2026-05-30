use std::{env, hint::black_box, time::Instant};

use vogon_core::{ModelAdapter, Result, Runtime, Step, StepId, Workflow};

#[derive(Debug, Clone)]
struct BenchModel;

impl ModelAdapter for BenchModel {
    fn complete(&self, step: &Step, input: &str) -> Result<String> {
        Ok(format!("{}:{input}", step.id().as_str()))
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let iterations = parse_iterations();
    let workflow = Workflow::new(
        "benchmark",
        vec![
            Step::new(StepId::new("classify")?, "Classify this support request."),
            Step::new(StepId::new("summarize")?, "Summarize the request."),
            Step::new(StepId::new("draft")?, "Draft a response."),
            Step::new(StepId::new("review")?, "Review the response."),
        ],
    )?;
    let runtime = Runtime::new(BenchModel);

    let started = Instant::now();
    for _ in 0..iterations {
        let report = runtime.run(black_box(&workflow))?;
        black_box(report);
    }
    let elapsed = started.elapsed();

    println!("iterations: {iterations}");
    println!("elapsed_ms: {}", elapsed.as_secs_f64() * 1000.0);
    println!(
        "iterations_per_second: {}",
        iterations as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}

fn parse_iterations() -> usize {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--iterations" {
            let Some(value) = args.next() else {
                return default_iterations();
            };
            return value.parse().unwrap_or_else(|_| default_iterations());
        }
    }

    default_iterations()
}

fn default_iterations() -> usize {
    1_000
}
