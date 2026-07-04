#!/usr/bin/env python3
"""Validate security workflow coverage and hardening settings."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


WORKFLOW_REQUIREMENTS = {
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
        "ubuntu runner": "    runs-on: ubuntu-24.04",
        "job timeout": "    timeout-minutes: 10",
        "checkout action": "uses: actions/checkout@v7",
        "dependency review action": "uses: actions/dependency-review-action@v5",
        "high severity failure": "          fail-on-severity: high",
    },
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

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
