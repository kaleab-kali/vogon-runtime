import tempfile
import unittest
from pathlib import Path

from scripts import check_pr_template


class CheckPrTemplateTests(unittest.TestCase):
    def test_accepts_template_with_readme_checks_and_extra_commands(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "python scripts/check_docs_links.py --root ."],
                template_commands=[
                    "cargo test",
                    "python scripts/check_docs_links.py --root .",
                    "docker build --tag vogon-runtime:smoke .",
                ],
            )

            self.assertEqual(check_pr_template.check_repository(root), [])

    def test_reports_missing_template_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_docs(
                root,
                readme_commands=["cargo test", "cargo clippy -- -D warnings"],
                template_commands=["cargo test"],
            )

            errors = check_pr_template.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/pull_request_template.md: missing README local check `cargo clippy -- -D warnings`"
                ],
            )

    def test_reports_missing_command_blocks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".github").mkdir()
            (root / "README.md").write_text("# README\n", encoding="utf-8")
            (root / ".github" / "pull_request_template.md").write_text(
                "## Verification\n\n- [ ] Relevant CLI smoke test:\n",
                encoding="utf-8",
            )

            errors = check_pr_template.check_repository(root)

            self.assertEqual(
                errors,
                [
                    "README.md: missing local check command block",
                    ".github/pull_request_template.md: missing verification command checklist",
                ],
            )


def write_docs(
    root: Path,
    *,
    readme_commands: list[str],
    template_commands: list[str],
) -> None:
    (root / ".github").mkdir()
    (root / "README.md").write_text(
        "# README\n\nRun local checks:\n\n```sh\n"
        + "\n".join(readme_commands)
        + "\n```\n",
        encoding="utf-8",
    )
    checklist = "\n".join(f"- [ ] `{command}`" for command in template_commands)
    (root / ".github" / "pull_request_template.md").write_text(
        "## Verification\n\n" + checklist + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
