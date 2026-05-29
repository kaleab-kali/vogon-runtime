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
- Replay file writing with `vogon run --output`.
- Step result caching support in `vogon-core`.
- Example workflow and replay fixtures for support triage and writing workflows.
- GitHub Actions CI for formatting, linting, tests, and docs.
- Optimized release build validation in CI.
- Release CLI smoke testing in CI.
- Tag-triggered GitHub release workflow for Linux CLI artifacts.
- Compile-time unsafe Rust prohibition across workspace crates.
- Workspace crate package archive validation in CI.
- Dependabot update checks for Cargo dependencies and GitHub Actions.

### Documentation

- Public README quickstart and project overview.
- CLI command reference.
- Architecture, workflow format, determinism, and replay format notes.
- Crate-level Rustdoc examples for `vogon-core` and `vogon-adapters`.
- Broken intra-doc link checks for public library crate docs.
- Contributing, security, code of conduct, and license documents.
