import tempfile
import unittest
from pathlib import Path

from scripts import check_public_status_docs


class CheckPublicStatusDocsTests(unittest.TestCase):
    def test_accepts_current_status_docs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_status_docs(root)

            self.assertEqual(check_public_status_docs.check_repository(root), [])

    def test_reports_missing_status_document(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_status_docs(root)
            (root / "SUPPORT.md").unlink()

            errors = check_public_status_docs.check_repository(root)

            self.assertIn("SUPPORT.md: missing public status document", errors)

    def test_reports_stale_pre_release_wording(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_status_docs(
                root,
                readme=(
                    "# README\n\n"
                    "Vogon Runtime is pre-release. The current codebase is a small Rust workspace.\n"
                ),
            )

            errors = check_public_status_docs.check_repository(root)

            self.assertIn(
                'README.md: stale status phrase "Vogon Runtime is pre-release"',
                errors,
            )

    def test_reports_missing_first_release_wording(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_status_docs(root, security="# Security\n\nSecurity fixes are handled.\n")

            errors = check_public_status_docs.check_repository(root)

            self.assertIn(
                'SECURITY.md: missing "`v0.1.1` is the latest public release of Vogon Runtime; `v0.1.0` was the first public release."',
                errors,
            )


def write_status_docs(
    root: Path,
    *,
    readme: str | None = None,
    security: str | None = None,
) -> None:
    (root / "docs").mkdir()
    (root / "README.md").write_text(
        readme
        or (
            "# README\n\n"
            "Vogon Runtime's latest public release is `v0.1.1`; `v0.1.0` was the first\n"
            "public release. The project is still in the `0.x` series, so command and\n"
            "library APIs may change as the runtime\n"
            "stabilizes.\n"
        ),
        encoding="utf-8",
    )
    (root / "SECURITY.md").write_text(
        security
        or (
            "# Security\n\n"
            "`v0.1.1` is the latest public release of Vogon Runtime; `v0.1.0` was the first\n"
            "public release. Security fixes are handled on the `main` branch and shipped in\n"
            "follow-up patch or minor releases when they affect published artifacts.\n"
        ),
        encoding="utf-8",
    )
    (root / "SUPPORT.md").write_text(
        "# Support\n\nVogon Runtime is released open-source software in the `0.x` series.\n",
        encoding="utf-8",
    )
    (root / "CHANGELOG.md").write_text(
        "# Changelog\n\n"
        "and this project follows semantic versioning.\n\n"
        "## [0.1.1] - 2026-07-08\n\n"
        "## [0.1.0] - 2026-07-08\n",
        encoding="utf-8",
    )
    (root / "docs" / "release.md").write_text(
        "# Release\n\nCrate publishing is manual while still in the `0.x` series.\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
