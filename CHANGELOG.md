# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

### Added

- Initial Rust workspace scaffold for `vogon-core`, `vogon-adapters`, and `vogon-cli`.
- Deterministic workflow execution with replay reports and verification.
- TOML workflow loading for CLI runs and validation.
- CLI commands for `demo`, `check`, `run`, `verify`, and `trace`.
- Replay JSONL trace export for machine-readable diagnostics.
- Literal replay redaction support for known sensitive values.
- Trace output redaction for known sensitive values.
- Replay file writing with `vogon run --output`.
- Temporary-file replay writes for more reliable `vogon run --output` updates.
- Validation for blank workflow step prompts.
- Step result caching support in `vogon-core`.
- Example workflow and replay fixtures for support triage and writing workflows.
- GitHub Actions CI for formatting, linting, tests, and docs.
- Optimized release build validation in CI.
- Release CLI smoke testing in CI.
- Offline cargo install smoke testing for the CLI in CI.
- Tag-triggered GitHub release workflow for Linux CLI artifacts.
- Compile-time unsafe Rust prohibition across workspace crates.
- Pull request dependency review for high-or-critical vulnerable dependency changes.
- Workspace crate package archive validation in CI.
- Dependabot update checks for Cargo dependencies and GitHub Actions.
- Runtime benchmark smoke target and CI check.

### Documentation

- Public README quickstart and project overview.
- CLI command reference.
- Architecture, workflow format, determinism, and replay format notes.
- Crate-level Rustdoc examples for `vogon-core` and `vogon-adapters`.
- Broken intra-doc link checks for public library crate docs.
- CLI regression coverage for replay verification mismatches.
- CLI file-read errors include the affected workflow or replay path.
- Performance benchmarking guide.
- Contributing, security, code of conduct, and license documents.
- Public contributor guidance for protected `main` checks and merge commits.
