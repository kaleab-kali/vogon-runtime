# Security Policy

## Supported Versions

Vogon Runtime has not published a stable release yet. Security fixes are handled
on the `main` branch until `v0.1.0` is tagged.

## Reporting a Vulnerability

Please report vulnerabilities through GitHub private vulnerability reporting
when it is enabled for the repository. If that is unavailable, open a minimal
public issue that describes the affected area without exploit details.

Do not include secrets, API keys, private prompts, replay logs with sensitive
inputs, or customer data in public reports.

## Unsafe Code

Workspace crates forbid unsafe Rust. Changes that require unsafe code should be
discussed in an issue before implementation and must explain why a safe
alternative is not sufficient.
