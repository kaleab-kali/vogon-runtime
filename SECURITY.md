# Security Policy

## Supported Versions

`v0.1.4` is the latest public release of Vogon Runtime; `v0.1.0` was the first
public release. Security fixes are handled on the `main` branch and shipped in
follow-up patch or minor releases when they affect published artifacts.

## Reporting a Vulnerability

Please report vulnerabilities through GitHub private vulnerability reporting
when it is enabled for the repository. If that is unavailable, open a minimal
public issue that describes the affected area without exploit details.

Do not include secrets, API keys, private prompts, replay logs with sensitive
inputs, or customer data in public reports.

## Dependency Review

Pull requests are checked with GitHub Dependency Review. Dependency changes that
introduce high-or-critical known vulnerabilities should be fixed before merge.

The committed `Cargo.lock` is also audited against RustSec advisories on
dependency changes and on a weekly schedule. The audit can be run manually
before releases. New actionable advisories should be fixed or explicitly
documented before merge.

Dependabot is configured for Cargo dependencies, GitHub Actions, and Docker
base images so runtime and deployment dependency updates are visible as pull
requests. Dependency Review blocks high-severity dependency changes and enforces
the checked-in permissive-license policy in
`.github/dependency-review-config.yml`.

## Static Analysis

CodeQL scans Rust code on pull requests, pushes to `main`, a weekly schedule,
and manual maintainer runs. New actionable code scanning findings should be
fixed or explicitly documented before merge.

## Workflow Hardening

GitHub Actions workflows must use least-privilege top-level permissions,
explicit hosted runner versions, and bounded `timeout-minutes` values on every
job. Floating runner labels such as `ubuntu-latest` are avoided so CI and
release behavior changes deliberately. External actions must use explicit,
non-mutable refs; refs such as `main`, `master`, `latest`, and branch refs are
not allowed.

The container image must keep a small build context, avoid `latest` base image
tags, install runtime packages with `--no-install-recommends`, clean apt package
lists, and run as the non-root `vogon` user.

## Unsafe Code

Workspace crates forbid unsafe Rust. Changes that require unsafe code should be
discussed in an issue before implementation and must explain why a safe
alternative is not sufficient.
