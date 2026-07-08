#!/usr/bin/env python3
"""Validate security workflow coverage and hardening settings."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


WORKFLOW_REQUIREMENTS = {
    ".github/workflows/codeql.yml": {
        "workflow name": "name: CodeQL",
        "pull request trigger": "  pull_request:",
        "push main trigger": "  push:",
        "scheduled scan": '    - cron: "31 5 * * 2"',
        "manual dispatch trigger": "  workflow_dispatch:",
        "read-only contents permission": "  contents: read",
        "security events write permission": "  security-events: write",
        "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
        "stale run cancellation": "  cancel-in-progress: true",
        "cargo network retry env": "env:\n  CARGO_NET_RETRY: 10",
        "ubuntu runner": "    runs-on: ubuntu-24.04",
        "job timeout": "    timeout-minutes: 30",
        "checkout action": "uses: actions/checkout@v7",
        "CodeQL init action": "uses: github/codeql-action/init@v4",
        "Rust language configuration": "          languages: rust",
        "no-build analysis mode": "          build-mode: none",
        "extended security queries": "          queries: security-extended,security-and-quality",
        "CodeQL analyze action": "uses: github/codeql-action/analyze@v4",
    },
    ".github/workflows/security-audit.yml": {
        "workflow name": "name: Security Audit",
        "pull request trigger": "  pull_request:",
        "push main trigger": "  push:",
        "scheduled audit": '    - cron: "17 4 * * 1"',
        "manual dispatch trigger": "  workflow_dispatch:",
        "read-only contents permission": "permissions:\n  contents: read",
        "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
        "dependency lockfile path": "      - Cargo.lock",
        "workspace manifest path": "      - Cargo.toml",
        "crate manifest path": '      - "crates/**/Cargo.toml"',
        "audit workflow path": "      - .github/workflows/security-audit.yml",
        "ubuntu runner": "    runs-on: ubuntu-24.04",
        "job timeout": "    timeout-minutes: 10",
        "checkout action": "uses: actions/checkout@v7",
        "RustSec audit action": "uses: actions-rust-lang/audit@v1",
        "no issue creation": "          createIssues: false",
    },
    ".github/workflows/dependency-review.yml": {
        "workflow name": "name: Dependency Review",
        "pull request trigger": "  pull_request:",
        "read-only contents permission": "permissions:\n  contents: read",
        "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
        "stale run cancellation": "  cancel-in-progress: true",
        "ubuntu runner": "    runs-on: ubuntu-24.04",
        "job timeout": "    timeout-minutes: 10",
        "checkout action": "uses: actions/checkout@v7",
        "dependency review action": "uses: actions/dependency-review-action@v5",
        "dependency review config file": (
            "          config-file: ./.github/dependency-review-config.yml"
        ),
    },
}

DEPENDENCY_REVIEW_CONFIG_REQUIREMENTS = {
    "high severity failure": "fail-on-severity: high",
    "license checks enabled": "license-check: true",
    "vulnerability checks enabled": "vulnerability-check: true",
    "license allowlist": "allow-licenses:",
    "Apache license allowed": "  - Apache-2.0",
    "BSD license allowed": "  - BSD-3-Clause",
    "CDLA permissive license allowed": "  - CDLA-Permissive-2.0",
    "ISC license allowed": "  - ISC",
    "MIT license allowed": "  - MIT",
    "Unicode license allowed": "  - Unicode-3.0",
    "Unlicense allowed": "  - Unlicense",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root to scan. Defaults to the current directory.",
    )
    args = parser.parse_args()

    errors = check_repository(args.root.resolve())
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_repository(root: Path) -> list[str]:
    errors: list[str] = []
    for relative_path, requirements in WORKFLOW_REQUIREMENTS.items():
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path}: missing security workflow")
            continue

        text = path.read_text(encoding="utf-8")
        for description, snippet in requirements.items():
            if snippet not in text:
                errors.append(f"{relative_path}: missing {description}")

    config_path = root / ".github" / "dependency-review-config.yml"
    if not config_path.exists():
        errors.append(
            ".github/dependency-review-config.yml: missing dependency review policy"
        )
    else:
        config_text = config_path.read_text(encoding="utf-8")
        for description, snippet in DEPENDENCY_REVIEW_CONFIG_REQUIREMENTS.items():
            if snippet not in config_text:
                errors.append(
                    f".github/dependency-review-config.yml: missing {description}"
                )

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
