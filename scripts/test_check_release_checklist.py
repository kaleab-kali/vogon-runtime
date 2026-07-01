import tempfile
import unittest
from pathlib import Path

from scripts import check_release_checklist


class CheckReleaseChecklistTests(unittest.TestCase):
    def test_accepts_release_doc_with_readme_checks_and_extra_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "python scripts/check_docs_links.py --root ."],
                release_commands=[
                    "cargo test",
                    "python scripts/check_docs_links.py --root .",
                    "docker build --tag vogon-runtime:smoke .",
                ],
            )

            self.assertEqual(check_release_checklist.check_repository(root), [])

    def test_reports_missing_release_doc_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "cargo clippy -- -D warnings"],
                release_commands=["cargo test"],
            )

            errors = check_release_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "docs/release.md: missing README local check `cargo clippy -- -D warnings`"
                ],
            )

    def test_reports_missing_command_blocks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            (root / "README.md").write_text("# README\n", encoding="utf-8")
            (root / "docs" / "release.md").write_text("# Release\n", encoding="utf-8")

            errors = check_release_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "README.md: missing local check command block",
                    "docs/release.md: missing release verification command block",
                ],
            )


def write_docs(
    root: Path,
    *,
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


if __name__ == "__main__":
    unittest.main()
