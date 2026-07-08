import tempfile
import unittest
from pathlib import Path

from scripts import check_ci_workflow


class CheckCiWorkflowTests(unittest.TestCase):
    def test_accepts_ci_workflow_contract(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_ci_workflow(root, ci_workflow_text())

            self.assertEqual(check_ci_workflow.check_repository(root), [])

    def test_reports_missing_ci_workflow(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_ci_workflow.check_repository(Path(directory))

            self.assertEqual(errors, [".github/workflows/ci.yml: missing CI workflow"])

    def test_reports_missing_required_ci_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_ci_workflow(
                root,
                ci_workflow_text().replace(
                    "python3 scripts/check_ci_workflow.py --root .",
                    "python3 scripts/check_other_workflow.py --root .",
                ),
            )

            errors = check_ci_workflow.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml: missing CI workflow validator",
                errors,
            )

    def test_reports_missing_required_occurrence_count(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_ci_workflow(
                root,
                ci_workflow_text().replace(
                    "uses: actions/checkout@v7",
                    "uses: actions/checkout@v6",
                    1,
                ),
            )

            errors = check_ci_workflow.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml: expected at least 4 occurrence(s) of `uses: actions/checkout@v7`, found 3",
                errors,
            )


def write_ci_workflow(root: Path, text: str) -> None:
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    (workflows / "ci.yml").write_text(text, encoding="utf-8")


def ci_workflow_text() -> str:
    return """name: CI

on:
  pull_request:
  push:
    branches:
      - main

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_NET_RETRY: 10

jobs:
  rust:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          python3 -m unittest scripts.test_check_ci_workflow
          python3 scripts/check_ci_workflow.py --root .
          python3 scripts/check_workflow_policies.py --root .
          python3 scripts/check_security_workflows.py --root .
          python3 scripts/check_container_policy.py --root .
          python3 scripts/check_release_workflow.py --root .
          python3 -m unittest scripts.test_check_public_status_docs
          python3 scripts/check_public_status_docs.py --root .
          python3 scripts/check_live_workflows.py --root .
          python3 -m unittest scripts.test_check_deployment_docs
          python3 scripts/check_deployment_docs.py --root .
          cargo fmt --all -- --check
          cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
          cargo test --workspace --all-features --locked
          cargo check -p vogon-cli --no-default-features --locked
          cargo bench -p vogon-core --bench runtime --locked -- --iterations 100
          cargo build --release --workspace --all-features --locked
          ./target/release/vogon doctor --json
          ./target/release/vogon providers --json
          python3 -m unittest scripts.test_check_providers_json
          python3 scripts/check_providers_json.py
          ./target/release/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
          cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
          cargo package -p vogon-core --allow-dirty --offline --locked
          cargo package --workspace --allow-dirty --no-verify --offline --locked
      - env:
          RUSTDOCFLAGS: -D warnings
        run: cargo doc --workspace --all-features --no-deps --locked

  msrv:
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v7
      - run: cargo +1.85.0 test --workspace --all-features --locked

  container-smoke:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          docker build --tag vogon-runtime:ci .
          docker run --rm --read-only vogon-runtime:ci --version

  windows-release-smoke:
    runs-on: windows-2025-vs2026
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v7
      - run: |
          cargo build --release -p vogon-cli --locked
          .\\target\\release\\vogon.exe verify fixtures\\workflows\\support-triage.toml fixtures\\replays\\support-triage.replay.json
"""


if __name__ == "__main__":
    unittest.main()
