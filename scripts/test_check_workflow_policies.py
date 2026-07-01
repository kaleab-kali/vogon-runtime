import tempfile
import unittest
from pathlib import Path

from scripts import check_workflow_policies


class CheckWorkflowPoliciesTests(unittest.TestCase):
    def test_accepts_least_privilege_workflow_with_job_scoped_write(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "release.yml").write_text(
                "\n".join(
                    [
                        "name: Release",
                        "on:",
                        "  workflow_dispatch:",
                        "permissions:",
                        "  contents: read",
                        "jobs:",
                        "  publish:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: 10",
                        "    steps:",
                        "      - uses: actions/checkout@v7",
                        "      - uses: github/codeql-action/analyze@v4",
                        "      - uses: docker://alpine:3.20",
                        "      - uses: ./github/actions/local-check",
                        "    permissions:",
                        "      contents: write",
                    ]
                ),
                encoding="utf-8",
            )

            self.assertEqual(check_workflow_policies.check_repository(root), [])

    def test_reports_missing_top_level_permissions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(["name: CI", "on:", "  pull_request:", "jobs:"]),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertEqual(
                errors,
                [".github/workflows/ci.yml: missing top-level permissions block"],
            )

    def test_rejects_pull_request_target_and_broad_permissions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(
                    [
                        "name: CI",
                        "on:",
                        "  pull_request_target:",
                        "permissions: write-all",
                        "jobs:",
                        "  test:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: 10",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml:3: pull_request_target is not allowed",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:4: broad workflow permissions are not allowed",
                errors,
            )

    def test_rejects_top_level_write_permissions_except_security_events(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(
                    [
                        "name: CI",
                        "on:",
                        "  push:",
                        "permissions:",
                        "  contents: write",
                        "  security-events: write",
                        "jobs:",
                        "  test:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: 10",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml:5: top-level contents permission must be read",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:5: top-level contents write permission must be job-scoped",
                errors,
            )
            self.assertNotIn(
                ".github/workflows/ci.yml:6: top-level security-events write permission must be job-scoped",
                errors,
            )

    def test_rejects_floating_runner_and_missing_timeout(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(
                    [
                        "name: CI",
                        "on:",
                        "  pull_request:",
                        "permissions:",
                        "  contents: read",
                        "jobs:",
                        "  test:",
                        "    runs-on: ubuntu-latest",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml:8: job `test` uses floating runner `ubuntu-latest`",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:7: job `test` missing timeout-minutes",
                errors,
            )

    def test_rejects_invalid_timeout_values(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(
                    [
                        "name: CI",
                        "on:",
                        "  pull_request:",
                        "permissions:",
                        "  contents: read",
                        "jobs:",
                        "  slow:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: 90",
                        "  invalid:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: soon",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml:9: job `slow` timeout-minutes must be between 1 and 60",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:12: job `invalid` timeout-minutes must be an integer",
                errors,
            )

    def test_rejects_unpinned_and_mutable_action_refs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflows = root / ".github" / "workflows"
            workflows.mkdir(parents=True)
            (workflows / "ci.yml").write_text(
                "\n".join(
                    [
                        "name: CI",
                        "on:",
                        "  pull_request:",
                        "permissions:",
                        "  contents: read",
                        "jobs:",
                        "  test:",
                        "    runs-on: ubuntu-24.04",
                        "    timeout-minutes: 10",
                        "    steps:",
                        "      - uses: actions/checkout",
                        "      - uses: github/codeql-action/analyze@main",
                        "      - uses: actions/cache@refs/heads/main",
                        "      - uses: actions/upload-artifact@${{ inputs.ref }}",
                    ]
                ),
                encoding="utf-8",
            )

            errors = check_workflow_policies.check_repository(root)

            self.assertIn(
                ".github/workflows/ci.yml:11: external action references must include an explicit ref",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:12: action reference `github/codeql-action/analyze@main` uses a mutable ref",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:13: action reference `actions/cache@refs/heads/main` uses a mutable ref",
                errors,
            )
            self.assertIn(
                ".github/workflows/ci.yml:14: action references must not use expressions",
                errors,
            )


if __name__ == "__main__":
    unittest.main()
