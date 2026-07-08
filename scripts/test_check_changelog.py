import tempfile
import unittest
from pathlib import Path

from scripts import check_changelog


class CheckChangelogTests(unittest.TestCase):
    def test_accepts_valid_changelog(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_changelog(
                root,
                """
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

### Added

- Initial feature.
""".strip()
                + "\n",
            )

            self.assertEqual(check_changelog.check_repository(root), [])

    def test_accepts_empty_unreleased_after_dated_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_changelog(
                root,
                """
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

## [0.1.0] - 2026-07-08

### Added

- Initial feature.
""".strip()
                + "\n",
            )

            self.assertEqual(check_changelog.check_repository(root), [])

    def test_reports_missing_required_structure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_changelog(root, "# Changes\n\n## Next\n")

            errors = check_changelog.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "CHANGELOG.md: first line must be `# Changelog`",
                    "CHANGELOG.md: missing Keep a Changelog 1.1.0 reference",
                    "CHANGELOG.md: missing semantic versioning note",
                    "CHANGELOG.md: missing `## [Unreleased]` section",
                ],
            )

    def test_reports_empty_and_unsupported_unreleased_subsections(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_changelog(
                root,
                """
# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

### Internal

### Fixed

## [0.1.0] - 2026-07-08
""".strip()
                + "\n",
            )

            errors = check_changelog.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "CHANGELOG.md: unsupported Unreleased subsection `Internal`",
                    "CHANGELOG.md: Unreleased `Internal` subsection has no entries",
                    "CHANGELOG.md: Unreleased `Fixed` subsection has no entries",
                ],
            )

    def test_reports_release_heading_without_date(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_changelog(
                root,
                """
# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once the first release is tagged.

## [Unreleased]

## [0.1.0]

### Added

- Initial feature.
""".strip()
                + "\n",
            )

            errors = check_changelog.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "CHANGELOG.md: release heading `## [0.1.0]` must include a version and date",
                ],
            )


def write_changelog(root: Path, text: str) -> None:
    (root / "CHANGELOG.md").write_text(text, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
