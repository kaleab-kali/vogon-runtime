import tempfile
import unittest
from pathlib import Path

from scripts import check_package_verification_docs


class CheckPackageVerificationDocsTests(unittest.TestCase):
    def test_accepts_documented_package_verification_rationale(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(root)

            self.assertEqual(
                check_package_verification_docs.check_repository(root),
                [],
            )

    def test_reports_missing_package_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(root, package_command="cargo package --workspace --offline")

            errors = check_package_verification_docs.check_repository(root)

            self.assertIn("README.md: missing offline package command", errors)
            self.assertIn("docs/release.md: missing offline package command", errors)

    def test_reports_missing_rationale(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(root, rationale="Run this after the other checks.")

            errors = check_package_verification_docs.check_repository(root)

            self.assertIn(
                "README.md: missing package verification rationale",
                errors,
            )
            self.assertIn(
                "docs/release.md: missing package verification rationale",
                errors,
            )


def write_docs(
    root: Path,
    *,
    package_command: str = check_package_verification_docs.PACKAGE_COMMAND,
    rationale: str | None = None,
) -> None:
    rationale = rationale or (
        "Cargo can fail offline verification while resolving unpublished internal "
        "workspace crates. The preceding build, test, docs, install, and smoke "
        "commands still verify compilation and CLI behavior."
    )
    (root / "docs").mkdir()
    text = f"{package_command}\n\n{rationale}\n"
    (root / "README.md").write_text(text, encoding="utf-8")
    (root / "docs" / "release.md").write_text(text, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
