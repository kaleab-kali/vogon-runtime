# Vogon Runtime

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
- Verification tools that compare a new run against a saved replay.
- Trace output for debugging and observability.

## Project Status

Vogon Runtime is pre-release. The current codebase is a small Rust workspace
with:

- `vogon-core` for workflow, runtime, replay, hashing, and error types.
- `vogon-adapters` for deterministic and provider-backed model adapters.
- `vogon-cli` for running demos, workflows, verification, and traces.
- `fixtures` for example workflows and replay logs.
- `docs` for architecture and replay format notes.

The first implementation will use a deterministic fake model before adding real
provider integrations.

## Quickstart

Run the deterministic demo workflow:

```sh
cargo run -p vogon-cli -- demo
```

Run a TOML workflow file:

```sh
cargo run -p vogon-cli -- run fixtures/workflows/support-triage.toml
```

Run local checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Repository Layout

```text
crates/
  vogon-core/      Workflow, runtime, replay, hashing, and errors.
  vogon-adapters/  Model adapter implementations.
  vogon-cli/       Command-line entrypoint.
docs/              Architecture and replay format notes.
examples/          Small API usage examples.
fixtures/          Example workflows and replay logs.
```

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
