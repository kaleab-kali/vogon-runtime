import tempfile
import unittest
from pathlib import Path

from scripts import check_contributing_checklist


class CheckContributingChecklistTests(unittest.TestCase):
    def test_accepts_contributing_doc_with_readme_checks_and_extra_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "python scripts/check_docs_links.py --root ."],
                contributing_commands=[
                    "cargo test",
                    "python scripts/check_docs_links.py --root .",
                    "docker build --tag vogon-runtime:smoke .",
                ],
                live_guidance=live_guidance_text(),
            )

            self.assertEqual(check_contributing_checklist.check_repository(root), [])

    def test_reports_missing_contributing_doc_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "cargo clippy -- -D warnings"],
                contributing_commands=["cargo test"],
                live_guidance=live_guidance_text(),
            )

            errors = check_contributing_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "CONTRIBUTING.md: missing README local check `cargo clippy -- -D warnings`"
                ],
            )

    def test_reports_missing_live_workflow_guidance(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test"],
                contributing_commands=["cargo test"],
                live_guidance=live_guidance_text().replace(
                    "- `Live OpenAI-Compatible Smoke` uses `OPENAI_COMPATIBLE_API_KEY`.\n",
                    "",
                ),
            )

            errors = check_contributing_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "CONTRIBUTING.md: missing `Live OpenAI-Compatible Smoke` guidance",
                    "CONTRIBUTING.md: missing `OPENAI_COMPATIBLE_API_KEY` live smoke secret guidance",
                ],
            )

    def test_reports_missing_command_blocks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("# README\n", encoding="utf-8")
            (root / "CONTRIBUTING.md").write_text(
                "# Contributing\n" + live_guidance_text(),
                encoding="utf-8",
            )

            errors = check_contributing_checklist.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "README.md: missing local check command block",
                    "CONTRIBUTING.md: missing development command block",
                ],
            )


def write_docs(
    root: Path,
    *,
    readme_commands: list[str],
    contributing_commands: list[str],
    live_guidance: str = "",
) -> None:
    (root / "README.md").write_text(
        "# README\n\nRun local checks:\n\n```sh\n"
        + "\n".join(readme_commands)
        + "\n```\n",
        encoding="utf-8",
    )
    (root / "CONTRIBUTING.md").write_text(
        "# Contributing\n\n## Development\n\n```sh\n"
        + "\n".join(contributing_commands)
        + "\n```\n"
        + live_guidance,
        encoding="utf-8",
    )


def live_guidance_text() -> str:
    return """
- `Live Gemini Smoke` uses `GEMINI_API_KEY`.
- `Live Groq Smoke` uses `GROQ_API_KEY`.
- `Live Hugging Face Smoke` uses `HF_TOKEN`.
- `Live OpenAI-Compatible Smoke` uses `OPENAI_COMPATIBLE_API_KEY`.
- `Live OpenRouter Smoke` uses `OPENROUTER_API_KEY`.
"""


if __name__ == "__main__":
    unittest.main()
