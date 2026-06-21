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


if __name__ == "__main__":
    unittest.main()
