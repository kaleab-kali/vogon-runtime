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
                    ".github/workflows/codeql.yml: missing security workflow",
                    ".github/workflows/security-audit.yml: missing security workflow",
                    ".github/workflows/dependency-review.yml: missing security workflow",
                    ".github/dependency-review-config.yml: missing dependency review policy",
                ],
            )

    def test_reports_missing_codeql_extended_queries(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                codeql=codeql_text().replace(
                    "          queries: security-extended,security-and-quality\n",
                    "",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [".github/workflows/codeql.yml: missing extended security queries"],
            )

    def test_reports_missing_rustsec_schedule(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                security_audit=security_audit_text().replace(
                    '    - cron: "17 4 * * 1"\n',
                    "",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [".github/workflows/security-audit.yml: missing scheduled audit"],
            )

    def test_reports_missing_dependency_review_config_reference(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                dependency_review=dependency_review_text().replace(
                    "          config-file: ./.github/dependency-review-config.yml",
                    "          fail-on-severity: high",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [
                    ".github/workflows/dependency-review.yml: missing dependency review config file"
                ],
            )

    def test_reports_missing_dependency_review_concurrency(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                dependency_review=dependency_review_text().replace(
                    "concurrency:\n"
                    "  group: ${{ github.workflow }}-${{ github.ref }}\n"
                    "  cancel-in-progress: true\n\n",
                    "",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [
                    ".github/workflows/dependency-review.yml: missing concurrency group",
                    ".github/workflows/dependency-review.yml: missing stale run cancellation",
                ],
            )

    def test_reports_disabled_dependency_review_license_check(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                dependency_review_config=dependency_review_config_text().replace(
                    "license-check: true",
                    "license-check: false",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [
                    ".github/dependency-review-config.yml: missing license checks enabled"
                ],
            )

    def test_reports_removed_dependency_review_license_allowlist_entry(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_security_workflows(
                root,
                dependency_review_config=dependency_review_config_text().replace(
                    "  - CDLA-Permissive-2.0\n",
                    "",
                ),
            )

            self.assertEqual(
                check_security_workflows.check_repository(root),
                [
                    ".github/dependency-review-config.yml: missing CDLA permissive license allowed"
                ],
            )


def write_security_workflows(
    root: Path,
    codeql: str | None = None,
    security_audit: str | None = None,
    dependency_review: str | None = None,
    dependency_review_config: str | None = None,
) -> None:
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    (workflows / "codeql.yml").write_text(
        codeql or codeql_text(),
        encoding="utf-8",
    )
    (workflows / "security-audit.yml").write_text(
        security_audit or security_audit_text(),
        encoding="utf-8",
    )
    (workflows / "dependency-review.yml").write_text(
        dependency_review or dependency_review_text(),
        encoding="utf-8",
    )
    (root / ".github" / "dependency-review-config.yml").write_text(
        dependency_review_config or dependency_review_config_text(),
        encoding="utf-8",
    )


def codeql_text() -> str:
    return """name: CodeQL

on:
  pull_request:
  push:
    branches:
      - main
  schedule:
    - cron: "31 5 * * 2"
  workflow_dispatch:

permissions:
  contents: read
  security-events: write

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_NET_RETRY: 10

jobs:
  analyze:
    name: CodeQL Rust analysis
    runs-on: ubuntu-24.04
    timeout-minutes: 30

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Initialize CodeQL
        uses: github/codeql-action/init@v4
        with:
          languages: rust
          build-mode: none
          queries: security-extended,security-and-quality

      - name: Perform CodeQL analysis
        uses: github/codeql-action/analyze@v4
"""


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

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

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
          config-file: ./.github/dependency-review-config.yml
"""


def dependency_review_config_text() -> str:
    return """fail-on-severity: high
license-check: true
vulnerability-check: true
allow-licenses:
  - Apache-2.0
  - BSD-3-Clause
  - CDLA-Permissive-2.0
  - ISC
  - MIT
  - Unicode-3.0
  - Unlicense
"""


if __name__ == "__main__":
    unittest.main()
