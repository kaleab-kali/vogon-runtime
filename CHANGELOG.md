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
- Release CLI smoke testing against every committed replay fixture.
- Windows release CLI smoke testing in CI.
- Offline cargo install smoke testing for the CLI in CI.
- Tag-triggered GitHub release workflow for Linux CLI artifacts.
- Tag-triggered GitHub release workflow for Windows CLI artifacts.
- Manual release workflow dry runs without publishing a GitHub release.
- SHA-256 checksum files for release archives.
- Release jobs now smoke test packaged archives after extraction.
- Compile-time unsafe Rust prohibition across workspace crates.
- Pull request dependency review for high-or-critical vulnerable dependency changes.
- Workspace crate package archive validation in CI.
- Dependabot update checks for Cargo dependencies and GitHub Actions.
- Runtime benchmark smoke target and CI check.
- CLI verification safety checks for redacted replay labels.
- Minimum supported Rust version CI testing for Rust 1.85.0.

### Changed

- Redacted replay mismatch reports now mask actual step output values.
- Release workflow artifact upload and download actions now use Node.js 24
  action versions.
- Release workflow token permissions are read-only except for the tag
  publishing job.
- CI now treats Rustdoc warnings as errors.
- Linux CI and release jobs now use the explicit `ubuntu-24.04` runner label.
- Windows CI and release jobs now use the explicit `windows-2025-vs2026`
  runner label.
- CI package validation now uses the offline package command documented for
  contributors.
- `vogon run --output` errors now include the affected replay output path or
  parent directory.
- `vogon verify` now rejects malformed redaction markers before executing a
  workflow.
- Redaction now applies longer overlapping literals before shorter ones to
  avoid partial secret exposure.
- Workflow step IDs with leading or trailing whitespace are now rejected instead
  of silently normalized.

### Documentation

- Public README quickstart and project overview.
- CLI command reference.
- Architecture, workflow format, determinism, and replay format notes.
- Crate-level Rustdoc examples for `vogon-core` and `vogon-adapters`.
- Broken intra-doc link checks for public library crate docs.
- CLI regression coverage for replay verification mismatches.
- CLI regression coverage for verifying redacted replay files.
- CLI regression coverage for redacted replay safety failures.
- CLI regression coverage for malformed workflow and replay parse errors.
- CLI file-read errors include the affected workflow or replay path.
- Redacted replay verification safety notes in README, CLI reference, and
  replay format documentation.
- Contributor and release verification checklists include MSRV and benchmark
  checks.
- README requirements list the minimum supported Rust version.
- README local verification commands match the enforced CI and release checks.
- README roadmap now distinguishes shipped runtime capabilities from planned
  provider and deployment work.
- Release documentation explains how to verify downloaded archive checksums.
- Performance benchmarking guide.
- Contributing, security, code of conduct, and license documents.
- Public contributor guidance for protected `main` checks and merge commits.
- Crate package metadata includes discovery keywords and crates.io categories.
- Blank public issues are disabled so reports use the guided issue templates or
  private vulnerability reporting.
- Contributor, pull request, and release verification docs use the offline
  package validation command.
