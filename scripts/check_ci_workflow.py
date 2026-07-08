#!/usr/bin/env python3
"""Validate required CI workflow jobs, policy checks, and smoke commands."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


REQUIRED_SNIPPETS = {
    "workflow name": "name: CI",
    "pull request trigger": "  pull_request:",
    "push main trigger": "  push:\n    branches:\n      - main",
    "read-only contents permission": "permissions:\n  contents: read",
    "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
    "stale run cancellation": "  cancel-in-progress: true",
    "cargo network retry env": "env:\n  CARGO_NET_RETRY: 10",
    "Rust workspace job": "  rust:",
    "MSRV job": "  msrv:",
    "container smoke job": "  container-smoke:",
    "Windows release smoke job": "  windows-release-smoke:",
    "Rust workspace runner": "    runs-on: ubuntu-24.04",
    "Windows runner": "    runs-on: windows-2025-vs2026",
    "Rust workspace timeout": "    timeout-minutes: 30",
    "MSRV timeout": "    timeout-minutes: 20",
    "checkout action": "uses: actions/checkout@v7",
    "CI workflow validator unit test": "python3 -m unittest scripts.test_check_ci_workflow",
    "CI workflow validator": "python3 scripts/check_ci_workflow.py --root .",
    "workflow policy validator": "python3 scripts/check_workflow_policies.py --root .",
    "security workflow validator": "python3 scripts/check_security_workflows.py --root .",
    "container policy validator": "cargo run -p vogon-xtask -- check-container-policy --root .",
    "committed secret validator": "cargo run -p vogon-xtask -- check-secrets --root .",
    "release workflow validator": "python3 scripts/check_release_workflow.py --root .",
    "changelog validator": "cargo run -p vogon-xtask -- check-changelog --root .",
    "contributing checklist validator": (
        "cargo run -p vogon-xtask -- check-contributing-checklist --root ."
    ),
    "deployment checklist validator": (
        "cargo run -p vogon-xtask -- check-deployment-checklist --root ."
    ),
    "deployment docs validator": (
        "cargo run -p vogon-xtask -- check-deployment-docs --root ."
    ),
    "pull request template validator": (
        "cargo run -p vogon-xtask -- check-pr-template --root ."
    ),
    "documentation link checker": (
        "cargo run -p vogon-xtask -- check-docs-links --root ."
    ),
    "issue template validator": (
        "cargo run -p vogon-xtask -- check-issue-templates --root ."
    ),
    "release checklist validator": (
        "cargo run -p vogon-xtask -- check-release-checklist --root ."
    ),
    "Cargo manifest validator": (
        "cargo run -p vogon-xtask -- check-cargo-manifests --root ."
    ),
    "provider env example validator": (
        "cargo run -p vogon-xtask -- check-env-example --root ."
    ),
    "Dependabot configuration validator": (
        "cargo run -p vogon-xtask -- check-dependabot-config --root ."
    ),
    "public status docs validator": (
        "cargo run -p vogon-xtask -- check-public-status-docs --root ."
    ),
    "package verification docs validator": (
        "cargo run -p vogon-xtask -- check-package-verification-docs --root ."
    ),
    "live workflow validator": "python3 scripts/check_live_workflows.py --root .",
    "format check": "cargo fmt --all -- --check",
    "clippy check": "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
    "workspace tests": "cargo test --workspace --all-features --locked",
    "deterministic-only CLI build": "cargo check -p vogon-cli --no-default-features --locked",
    "MSRV test": "cargo +1.85.0 test --workspace --all-features --locked",
    "benchmark smoke": "cargo bench -p vogon-core --bench runtime --locked -- --iterations 100",
    "release build": "cargo build --release --workspace --all-features --locked",
    "release CLI doctor smoke": "./target/release/vogon doctor --json",
    "release CLI providers smoke": "./target/release/vogon providers --json",
    "providers JSON validator unit test": "python3 -m unittest scripts.test_check_providers_json",
    "providers JSON validator": "scripts/check_providers_json.py",
    "release replay verification smoke": "./target/release/vogon verify",
    "offline install smoke": "cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force",
    "rustdoc warnings denied": "RUSTDOCFLAGS: -D warnings",
    "core package verification": "cargo package -p vogon-core --allow-dirty --offline --locked",
    "workspace package smoke": "cargo package --workspace --allow-dirty --no-verify --offline --locked",
    "container build smoke": "docker build --tag vogon-runtime:ci .",
    "read-only container smoke": "docker run --rm --read-only",
    "Windows release build": "cargo build --release -p vogon-cli --locked",
    "Windows replay verification smoke": ".\\target\\release\\vogon.exe verify",
}

REQUIRED_COUNTS = {
    "uses: actions/checkout@v7": 4,
    "runs-on: ubuntu-24.04": 3,
    "timeout-minutes: 30": 3,
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
    path = root / ".github" / "workflows" / "ci.yml"
    if not path.exists():
        return [".github/workflows/ci.yml: missing CI workflow"]

    text = path.read_text(encoding="utf-8")
    errors = [
        f".github/workflows/ci.yml: missing {description}"
        for description, snippet in REQUIRED_SNIPPETS.items()
        if snippet not in text
    ]

    for snippet, expected_count in REQUIRED_COUNTS.items():
        actual_count = text.count(snippet)
        if actual_count < expected_count:
            errors.append(
                ".github/workflows/ci.yml: "
                f"expected at least {expected_count} occurrence(s) of `{snippet}`, "
                f"found {actual_count}"
            )

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
