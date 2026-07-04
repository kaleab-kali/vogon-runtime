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
- CLI provider diagnostics with `vogon providers`.
- Replay JSONL trace export for machine-readable diagnostics.
- JSON workflow validation summaries with `vogon check --json`.
- JSON replay verification reports with `vogon verify --json`.
- `vogon verify --json` reports now include an explicit `is_match` field.
- Literal replay redaction support for known sensitive values.
- Trace output redaction for known sensitive values.
- Replay file writing with `vogon run --output`.
- Temporary-file replay writes for more reliable `vogon run --output` updates.
- Validation for blank workflow step prompts.
- Step result caching support in `vogon-core`.
- Example workflow and replay fixtures for support triage and writing workflows.
- GitHub Actions CI for formatting, linting, tests, and docs.
- GitHub issue forms for bug reports and feature requests.
- Public support guidance for questions, bugs, feature requests, and security
  reports.
- Documentation link checking in CI for repository-local Markdown links.
- Provider credential `.env.example` validation in CI.
- Conservative committed secret pattern scanning in CI.
- Optimized release build validation in CI.
- Release CLI smoke testing in CI.
- Release CLI smoke testing against every committed replay fixture.
- Release CLI smoke testing for machine-readable `check` and `verify` output.
- Release CLI smoke testing now asserts machine-readable `check` and `verify`
  JSON fields.
- Trace JSONL smoke checks now validate run and step event structure through a
  shared script.
- Workflow check JSON smoke checks now validate workflow names and step counts
  through a shared script.
- Replay verification JSON smoke checks now validate match status and mismatch
  shape through a shared script.
- Cache file smoke checks now validate entry bounds and insertion-order shape
  through a shared script.
- Release metadata and SPDX SBOM smoke checks now validate artifact JSON shape
  through shared scripts.
- Windows release CLI smoke testing in CI.
- Offline cargo install smoke testing for the CLI in CI.
- Offline cargo install smoke testing now verifies installed CLI workflow and
  replay behavior.
- Gemini API adapter support for opt-in real provider-backed workflow runs.
- Gemini API requests are bounded by a configurable `vogon run
  --gemini-timeout-seconds` timeout.
- Gemini API transient failures are retried by a configurable `vogon run
  --gemini-max-retries` count.
- Gemini API HTTP error bodies are capped before being returned in adapter
  errors.
- OpenAI-compatible chat-completions adapter for provider-backed workflow runs
  against configurable compatible endpoints.
- Groq provider preset for Groq's OpenAI-compatible chat-completions endpoint.
- Hugging Face provider preset for Hugging Face Inference Providers'
  OpenAI-compatible endpoint.
- OpenRouter provider preset for OpenRouter's OpenAI-compatible endpoint.
- Manual live Gemini provider smoke workflow for maintainers with
  `GEMINI_API_KEY` configured in GitHub Actions.
- Manual live OpenAI-compatible provider smoke workflow for maintainers with
  `OPENAI_COMPATIBLE_API_KEY` configured in GitHub Actions.
- Manual live Groq provider smoke workflow for maintainers with `GROQ_API_KEY`
  configured in GitHub Actions.
- Manual live Hugging Face provider smoke workflow for maintainers with
  `HF_TOKEN` configured in GitHub Actions.
- Manual live OpenRouter provider smoke workflow for maintainers with
  `OPENROUTER_API_KEY` configured in GitHub Actions.
- Dockerfile and deployment documentation for CLI container image builds.
- CI container image smoke testing for the CLI.
- Container smoke tests now assert the unprivileged runtime UID and read-only
  deterministic CLI execution.
- Tag-triggered GitHub release workflow for Linux CLI artifacts.
- Tag-triggered GitHub release workflow for Windows CLI artifacts.
- Release archives now include `README.md` and `LICENSE`.
- Manual release workflow dry runs without publishing a GitHub release.
- SHA-256 checksum files for release archives.
- Release jobs now smoke test packaged archives after extraction.
- Manual release workflow dry runs now download uploaded archives and verify
  their checksum files without publishing a GitHub release.
- Release jobs now smoke test machine-readable `check` and `verify` output.
- Release jobs now generate provenance attestations for packaged archives.
- Release jobs now publish locked Cargo dependency metadata with a checksum.
- Release jobs now publish SPDX dependency SBOMs with checksums.
- Release jobs now build, smoke test, checksum, attest, and upload a container
  image archive.
- Compile-time unsafe Rust prohibition across workspace crates.
- Pull request dependency review for high-or-critical vulnerable dependency changes.
- CodeQL static analysis for Rust code on pull requests, pushes, schedules, and
  manual maintainer runs.
- Workspace crate package archive validation in CI.
- RustSec advisory auditing for committed Rust dependencies.
- Dependabot update checks for Cargo dependencies and GitHub Actions.
- Runtime benchmark smoke target and CI check.
- Persistent `vogon run --cache-file` support for bounded provider output caches.
- Committed secret scanning now rejects tracked `*.cache.json` cache artifacts.
- CLI verification safety checks for redacted replay labels.
- Minimum supported Rust version CI testing for Rust 1.85.0.
- Cargo manifest metadata validation for open-source package readiness.
- Changelog structure validation in CI.
- Contributor verification checklist validation in CI.
- Deployment smoke checklist validation in CI.
- Release and installed CLI smoke checks now verify bounded run cache files.
- Container build context policy now excludes persistent run cache artifacts.
- Package verification rationale validation in CI.
- Contributor verification docs now include package verification rationale checks.
- Free and low-cost real provider path guidance.
- Provider diagnostics now include public usage and rate-limit links.
- Doctor diagnostics now print provider documentation, defaults, and usage links
  in human-readable output.
- Runtime observer events now report cache hit and miss status for cached
  workflow runs.
- Release and container smoke checks now validate provider usage links in
  `doctor --json` output.
- Manual release artifact smoke checks now check out repository scripts before
  validating downloaded artifacts.
- Live provider smoke workflow validation now requires checkout, toolchain, and
  release CLI build steps.
- Repository agent instructions now preserve merged branches unless deletion is
  explicitly requested.
- Issue template validation in CI.
- Pull request verification checklist validation in CI.

### Changed

- GitHub issue intake now uses structured issue forms instead of duplicate
  Markdown templates.
- Redacted replay mismatch reports now mask actual step output values.
- Release workflow artifact upload and download actions now use Node.js 24
  action versions.
- GitHub Actions checkout steps now use `actions/checkout@v7`.
- HTTP provider adapters now use `ureq` 3.3.
- Release workflow token permissions are read-only except for the tag
  publishing job.
- Provider retry counts are capped at 20 attempts at CLI and adapter
  construction boundaries.
- Redaction marker parsing now ignores escaped marker-like text and ordinary
  prose fragments that are not complete markers.
- Verification mismatch reports now redact both expected and actual step output
  values before printing human-readable or JSON output.
- CLI workflow and replay file reads now reject inputs larger than 1 MiB before
  buffering them into memory.
- `RunCache` now has a bounded entry limit with explicit output removal and
  clearing APIs.
- Provider adapters now apply exponential backoff with lightweight jitter before
  retrying transient failures.
- Runtime cache keys now include adapter cache identity so cached outputs are
  scoped by provider, endpoint, model, and adapter implementation.
- Replay reports now include schema version and non-secret runtime metadata for
  provider, adapter, model, cache identity, and runtime parameters.
- Provider diagnostics now include non-secret documentation URLs for each
  provider.
- `vogon verify` now defaults to replay provider metadata and reports runtime
  metadata mismatches for current-schema replays.
- GitHub Actions jobs now have explicit timeout limits.
- CI now treats Rustdoc warnings as errors.
- Library crates now deny missing public Rust documentation.
- CI now checks the CLI with default features disabled to preserve the
  deterministic-only build path.
- CI and release smoke tests now assert trace JSONL replay schema and runtime
  metadata output.
- Live provider smoke workflows now assert replay runtime metadata and check
  that configured API keys are absent from serialized replays.
- Cargo-based GitHub Actions workflows now increase Cargo registry retry
  attempts to reduce transient dependency fetch failures.
- Linux CI and release jobs now use the explicit `ubuntu-24.04` runner label.
- Windows CI and release jobs now use the explicit `windows-2025-vs2026`
  runner label.
- CI package validation now uses the offline package command documented for
  contributors.
- `vogon run --output` errors now include the affected replay output path or
  parent directory.
- `vogon run --output` now rejects directory output paths with an explicit
  error.
- `vogon verify` now rejects malformed redaction markers before executing a
  workflow.
- `vogon trace` now rejects malformed redaction markers before printing replay
  output.
- Replay commands now reject redaction markers with unsupported labels before
  verification or trace output.
- Redaction now applies longer overlapping literals before shorter ones to
  avoid partial secret exposure.
- Redaction labels with leading or trailing whitespace are now rejected instead
  of silently normalized.
- CLI redaction labels are now rejected when repeated in the same command.
- Core redaction sets now reject duplicate labels for library callers.
- Provider adapter HTTP error handling now reads only a bounded response-body
  prefix before truncating error messages.
- Workflow names with leading or trailing whitespace are now rejected.
- Workflow names with spaces or punctuation are now rejected.
- Workflow step IDs with leading or trailing whitespace are now rejected instead
  of silently normalized.
- Workflow TOML parsing now rejects unknown top-level and step fields.
- Workflow deserialization now rejects invalid workflow state for library
  callers.
- Replay JSON parsing now rejects unknown top-level and step fields.
- Replay JSON parsing now rejects malformed workflow names.
- Replay JSON parsing now rejects empty step lists.
- Replay JSON parsing now rejects duplicate step IDs.
- Replay JSON parsing now rejects malformed hash fields.

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
- CLI regression coverage for duplicate redaction labels across `run`, `verify`,
  and `trace`.
- CLI file-read errors include the affected workflow or replay path.
- Redacted replay verification safety notes in README, CLI reference, and
  replay format documentation.
- Contributor and release verification checklists include MSRV and benchmark
  checks.
- Contributor and release verification checklists include machine-readable CLI
  smoke checks.
- Contributor and release verification checklists include installed CLI workflow
  and replay smoke checks.
- README requirements list the minimum supported Rust version.
- README local verification commands match the enforced CI and release checks.
- README roadmap now distinguishes shipped runtime capabilities from planned
  provider and deployment work.
- Release documentation explains how to verify downloaded archive checksums.
- Provider adapter documentation records the Gemini integration, deterministic
  test path, and future free or low-cost provider candidates.
- Performance benchmarking guide.
- Contributing, security, code of conduct, and license documents.
- Public contributor guidance for protected `main` checks and merge commits.
- Crate package metadata includes discovery keywords and crates.io categories.
- Blank public issues are disabled so reports use the guided issue templates or
  private vulnerability reporting.
- Contributor, pull request, and release verification docs use the offline
  package validation command.
- README project status now clearly separates shipped deterministic adapters
  from planned provider integrations.
- Pull request template replay verification now includes every committed replay
  fixture.
- Security policy and contributor docs now describe RustSec advisory auditing.
- Contributor verification docs now stay aligned with the README local check
  list through CI validation.
- Deployment smoke commands now stay aligned with README and release
  verification docs through CI validation.
- Issue forms now stay aligned with required reproduction, security, and intake
  fields through CI validation.
- The pull request template verification checklist now stays aligned with the
  README local check list through CI validation.
