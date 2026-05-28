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
- GitHub Actions CI for formatting, linting, and tests.
- Workspace crate package archive validation in CI.

### Documentation

- Public README quickstart and project overview.
- Architecture, determinism, and replay format notes.
- Contributing, security, code of conduct, and license documents.
