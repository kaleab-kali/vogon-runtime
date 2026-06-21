import tempfile
import unittest
from pathlib import Path

from scripts import check_docs_links


class CheckDocsLinksTests(unittest.TestCase):
    def test_extracts_nested_badge_link_without_image_target(self):
        links = check_docs_links.markdown_link_targets(
            "[![CI](https://example.com/badge.svg)](https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml)"
        )

        self.assertEqual(
            links,
            ["https://github.com/kaleab-kali/vogon-runtime/actions/workflows/ci.yml"],
        )

    def test_accepts_relative_absolute_and_repo_blob_links(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            (root / "docs" / "guide.md").write_text("# Guide\n", encoding="utf-8")
            (root / "README.md").write_text(
                "\n".join(
                    [
                        "[Guide](docs/guide.md)",
                        "[Root guide](/docs/guide.md)",
                        "[GitHub guide](https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/guide.md)",
                        "[Anchor](#local-heading)",
                        "[External](https://example.com/docs)",
                    ]
                ),
                encoding="utf-8",
            )

            self.assertEqual(check_docs_links.check_repository(root), [])

    def test_reports_missing_repository_link_targets(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "README.md").write_text("[Missing](docs/missing.md)\n", encoding="utf-8")

            errors = check_docs_links.check_repository(root)

            self.assertEqual(len(errors), 1)
            self.assertIn("README.md:1", errors[0])
            self.assertIn("docs/missing.md", errors[0])


if __name__ == "__main__":
    unittest.main()
