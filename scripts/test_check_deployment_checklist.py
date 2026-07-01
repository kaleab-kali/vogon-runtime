import tempfile
import unittest
from pathlib import Path

from scripts import check_deployment_checklist


class CheckDeploymentChecklistTests(unittest.TestCase):
    def test_accepts_docs_with_deployment_commands_in_readme_and_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                deployment_commands=[
                    "docker build --tag vogon-runtime:smoke .",
                    "docker run --rm vogon-runtime:smoke --version",
                ],
                readme_commands=[
                    "cargo test",
                    "docker build --tag vogon-runtime:smoke .",
                    "docker run --rm vogon-runtime:smoke --version",
                ],
                release_commands=[
                    "cargo test",
                    "docker build --tag vogon-runtime:smoke .",
                    "docker run --rm vogon-runtime:smoke --version",
                ],
            )

            self.assertEqual(check_deployment_checklist.check_repository(root), [])

    def test_reports_deployment_commands_missing_from_readme_and_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                deployment_commands=[
                    "docker build --tag vogon-runtime:smoke .",
                    "docker run --rm vogon-runtime:smoke --version",
                ],
                readme_commands=["docker build --tag vogon-runtime:smoke ."],
                release_commands=["cargo test"],
            )

            errors = check_deployment_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "docs/release.md: missing deployment smoke command `docker build --tag vogon-runtime:smoke .`",
                    "README.md: missing deployment smoke command `docker run --rm vogon-runtime:smoke --version`",
                    "docs/release.md: missing deployment smoke command `docker run --rm vogon-runtime:smoke --version`",
                ],
            )

    def test_reports_missing_command_blocks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            (root / "README.md").write_text("# README\n", encoding="utf-8")
            (root / "docs" / "release.md").write_text("# Release\n", encoding="utf-8")
            (root / "docs" / "deployment.md").write_text("# Deployment\n", encoding="utf-8")

            errors = check_deployment_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "README.md: missing local check command block",
                    "docs/release.md: missing release verification command block",
                    "docs/deployment.md: missing deployment smoke command block",
                ],
            )


def write_docs(
    root: Path,
    *,
    deployment_commands: list[str],
    readme_commands: list[str],
    release_commands: list[str],
) -> None:
    (root / "docs").mkdir()
    (root / "README.md").write_text(
        "# README\n\nRun local checks:\n\n```sh\n"
        + "\n".join(readme_commands)
        + "\n```\n",
        encoding="utf-8",
    )
    (root / "docs" / "release.md").write_text(
        "# Release\n\nRun the full local verification set:\n\n```sh\n"
        + "\n".join(release_commands)
        + "\n```\n",
        encoding="utf-8",
    )
    (root / "docs" / "deployment.md").write_text(
        "# Deployment\n\nBefore publishing or deploying an image, run:\n\n```sh\n"
        + "\n".join(deployment_commands)
        + "\n```\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
