import tempfile
import unittest
from pathlib import Path

from scripts import check_archive_contents


class CheckArchiveContentsTests(unittest.TestCase):
    def test_accepts_expected_linux_archive_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            (archive / "vogon").write_text("binary", encoding="utf-8")
            (archive / "README.md").write_text("readme", encoding="utf-8")
            (archive / "LICENSE").write_text("license", encoding="utf-8")

            self.assertEqual(
                check_archive_contents.check_directory(archive, binary="vogon"),
                [],
            )

    def test_accepts_expected_windows_archive_contents(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            (archive / "vogon.exe").write_text("binary", encoding="utf-8")
            (archive / "README.md").write_text("readme", encoding="utf-8")
            (archive / "LICENSE").write_text("license", encoding="utf-8")

            self.assertEqual(
                check_archive_contents.check_directory(archive, binary="vogon.exe"),
                [],
            )

    def test_accepts_custom_required_files(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            (archive / "vogon").write_text("binary", encoding="utf-8")
            (archive / "NOTICE").write_text("notice", encoding="utf-8")

            self.assertEqual(
                check_archive_contents.check_directory(
                    archive,
                    binary="vogon",
                    required_files=["NOTICE"],
                ),
                [],
            )

    def test_reports_missing_archive_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_archive_contents.check_directory(
                Path(directory) / "missing",
                binary="vogon",
            )

            self.assertEqual(len(errors), 1)
            self.assertTrue(
                errors[0].startswith(
                    "Archive directory is missing or is not a directory:"
                )
            )

    def test_reports_missing_binary_and_required_files(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)

            self.assertEqual(
                check_archive_contents.check_directory(archive, binary="vogon"),
                [
                    "Packaged archive binary is missing: vogon",
                    "Packaged archive required file is missing: README.md",
                    "Packaged archive required file is missing: LICENSE",
                ],
            )

    def test_reports_directories_where_files_are_expected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)
            (archive / "vogon").mkdir()
            (archive / "README.md").mkdir()
            (archive / "LICENSE").mkdir()

            self.assertEqual(
                check_archive_contents.check_directory(archive, binary="vogon"),
                [
                    "Packaged archive binary is not a regular file: vogon",
                    "Packaged archive required file is not a regular file: README.md",
                    "Packaged archive required file is not a regular file: LICENSE",
                ],
            )


if __name__ == "__main__":
    unittest.main()
