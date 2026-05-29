# Vogon Runtime

[![CI](https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml/badge.svg)](https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml)

Vogon Runtime is a Rust runtime for deterministic, replayable AI workflows.

The project is built around a simple idea: an AI workflow should produce an
artifact that can be inspected, verified, and replayed later. Instead of treating
prompt chains as opaque scripts, Vogon records each workflow step, its inputs,
its outputs, and stable hashes that make drift visible.

## Why This Exists

LLM applications often become hard to debug as soon as they include multiple
steps, provider calls, retries, tool outputs, and prompt changes. A result may
look correct today and become impossible to explain tomorrow.

Vogon Runtime aims to make those workflows easier to operate by providing:

- Ordered workflow execution.
- A clean model adapter boundary.
- Deterministic fake models for local development and tests.
- Replay logs that capture step-level inputs, outputs, and hashes.
- Literal redaction for known sensitive output values.
- Verification tools that compare a new run against a saved replay.
- Trace output for debugging and observability.

## Project Status

Vogon Runtime is pre-release. The current codebase is a small Rust workspace
with:

- `vogon-core` for workflow, runtime, replay, hashing, and error types.
- `vogon-adapters` for deterministic and provider-backed model adapters.
- `vogon-cli` for running demos, workflows, verification, and traces.
- `fixtures` for example workflows and replay logs.
- `docs` for architecture, workflow format, determinism, and replay format
  notes.

The current implementation uses a deterministic model adapter so workflows,
replays, and verification can be developed without network access. Provider
integrations are planned behind adapter boundaries.

## Quickstart

Run the deterministic demo workflow:

```sh
cargo run -p vogon-cli -- demo
```

Run a TOML workflow file:

```sh
cargo run -p vogon-cli -- run fixtures/workflows/support-triage.toml
```

Validate a TOML workflow without executing it:

```sh
cargo run -p vogon-cli -- check fixtures/workflows/support-triage.toml
```

Write a replay file:

```sh
cargo run -p vogon-cli -- run --output target/support-triage.replay.json fixtures/workflows/support-triage.toml
```

Redact known sensitive literals from replay outputs:

```sh
cargo run -p vogon-cli -- run --redact api_key=sk-test-123 fixtures/workflows/support-triage.toml
```

Verify a saved replay:

```sh
cargo run -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

Verify a multi-step writing workflow fixture:

```sh
cargo run -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
```

Inspect a replay trace:

```sh
cargo run -p vogon-cli -- trace fixtures/replays/support-triage.replay.json
```

Export a machine-readable JSON Lines trace:

```sh
cargo run -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json
```

Run local checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --release --workspace --all-features
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo doc --workspace --all-features --no-deps
cargo package --workspace --allow-dirty --no-verify
```

## Repository Layout

```text
crates/
  vogon-core/      Workflow, runtime, replay, hashing, and errors.
  vogon-adapters/  Model adapter implementations and runtime examples.
  vogon-cli/       Command-line entrypoint.
docs/              Architecture, format, and determinism notes.
fixtures/          Example workflows and replay logs.
```

Useful documentation:

- [Architecture](docs/architecture.md)
- [CLI reference](docs/cli.md)
- [Workflow format](docs/workflow-format.md)
- [Determinism](docs/determinism.md)
- [Replay format](docs/replay-format.md)

## Design Principles

- Determinism first: local tests and replay verification should not require a
  network call.
- Provider isolation: provider-specific code should stay outside the core
  runtime.
- Inspectable artifacts: replay logs should be readable and stable enough to
  debug.
- Small public surface: the runtime API should be clear before adding workflow
  graph support, caching, retries, or provider configuration.

## Roadmap

- Scaffold the Rust workspace and CLI.
- Implement ordered workflow execution.
- Add deterministic replay log generation.
- Add replay verification with structured mismatch errors.
- Add fixtures and examples that can be run by new contributors.
- Add observability events and trace export.
- Add provider-backed adapters behind feature flags.

## License

Vogon Runtime is released under the MIT License.
