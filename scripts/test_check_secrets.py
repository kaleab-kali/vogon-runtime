import tempfile
import unittest
from pathlib import Path
from unittest import mock

from scripts import check_secrets


class CheckSecretsTests(unittest.TestCase):
    def test_reports_secret_like_values(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            secret_file = root / "README.md"
            secret_file.write_text(
                "token=sk-" + "abcdefghijklmnopqrstuvwxyz\n",
                encoding="utf-8",
            )

            with mock.patch.object(check_secrets, "tracked_files", return_value=[secret_file]):
                findings = check_secrets.check_repository(root)

            self.assertEqual(findings, ["README.md:1: possible OpenAI API key"])

    def test_accepts_short_test_placeholders(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            test_file = root / "docs.md"
            test_file.write_text(
                "\n".join(
                    [
                        "api_key=sk-test-123",
                        "token=secret-key",
                        "OPENROUTER_API_KEY=",
                    ]
                ),
                encoding="utf-8",
            )

            with mock.patch.object(check_secrets, "tracked_files", return_value=[test_file]):
                findings = check_secrets.check_repository(root)

            self.assertEqual(findings, [])

    def test_skips_binary_and_large_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary_file = root / "image.bin"
            binary_file.write_bytes(b"\0sk-abcdefghijklmnopqrstuvwxyz")
            large_file = root / "large.txt"
            large_file.write_text("x" * (check_secrets.MAX_TEXT_BYTES + 1), encoding="utf-8")

            with mock.patch.object(
                check_secrets,
                "tracked_files",
                return_value=[binary_file, large_file],
            ):
                findings = check_secrets.check_repository(root)

            self.assertEqual(findings, [])


if __name__ == "__main__":
    unittest.main()
