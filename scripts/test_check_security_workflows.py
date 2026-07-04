import tempfile
import unittest
from pathlib import Path

from scripts import check_security_workflows


class CheckSecurityWorkflowsTests(unittest.TestCase):
    def test_accepts_expected_security_workflows(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(root)

            self.assertEqual(check_security_workflows.check_repository(root), [])

    def test_reports_missing_security_workflows(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_security_workflows.check_repository(Path(directory))

            self.assertEqual(
                errors,
                [
                    ".github/workflows/security-audit.yml: missing security workflow",
                    ".github/workflows/dependency-review.yml: missing security workflow",
                ],
            )

    def test_reports_missing_rustsec_schedule(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                security_audit_text().replace('    - cron: "17 4 * * 1"\n', ""),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [".github/workflows/security-audit.yml: missing scheduled audit"],
            )

    def test_reports_missing_dependency_review_severity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                dependency_review=dependency_review_text().replace(
                    "          fail-on-severity: high",
                    "          fail-on-severity: critical",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [
                    ".github/workflows/dependency-review.yml: missing high severity failure"
                ],
            )


def write_security_workflows(
    root: Path,
    security_audit: str | None = None,
    dependency_review: str | None = None,
) -> None:
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    (workflows / "security-audit.yml").write_text(
        security_audit or security_audit_text(),
        encoding="utf-8",
    )
    (workflows / "dependency-review.yml").write_text(
        dependency_review or dependency_review_text(),
        encoding="utf-8",
    )


def security_audit_text() -> str:
    return """name: Security Audit

on:
  pull_request:
    paths:
      - Cargo.lock
      - Cargo.toml
      - "crates/**/Cargo.toml"
      - .github/workflows/security-audit.yml
  push:
    branches:
      - main
    paths:
      - Cargo.lock
      - Cargo.toml
      - "crates/**/Cargo.toml"
      - .github/workflows/security-audit.yml
  schedule:
    - cron: "17 4 * * 1"
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  rustsec:
    name: RustSec advisory audit
    runs-on: ubuntu-24.04
    timeout-minutes: 10

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Audit Cargo.lock
        uses: actions-rust-lang/audit@v1
        with:
          createIssues: false
"""


def dependency_review_text() -> str:
    return """name: Dependency Review

on:
  pull_request:

permissions:
  contents: read

jobs:
  dependency-review:
    name: Dependency review
    runs-on: ubuntu-24.04
    timeout-minutes: 10

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Review dependency changes
        uses: actions/dependency-review-action@v5
        with:
          fail-on-severity: high
"""


if __name__ == "__main__":
    unittest.main()
