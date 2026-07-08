import tempfile
import unittest
from pathlib import Path

from scripts import check_deployment_docs


class CheckDeploymentDocsTests(unittest.TestCase):
    def test_accepts_all_provider_container_examples(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_deployment_doc(root, provider_credentials_section())

            self.assertEqual(check_deployment_docs.check_repository(root), [])

    def test_reports_missing_provider_credentials_section(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_deployment_doc(root, "## Runtime Notes\n")

            errors = check_deployment_docs.check_repository(root)

            self.assertEqual(
                errors,
                ["docs/deployment.md: missing Provider Credentials section"],
            )

    def test_reports_missing_provider_env_and_run_example(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_deployment_doc(
                root,
                provider_credentials_section().replace("-e GROQ_API_KEY", "-e OTHER_KEY").replace(
                    "--provider openrouter",
                    "--provider deterministic",
                ),
            )

            errors = check_deployment_docs.check_repository(root)

            self.assertIn(
                "docs/deployment.md: missing container env example for GROQ_API_KEY",
                errors,
            )
            self.assertIn(
                "docs/deployment.md: missing container run example for provider `openrouter`",
                errors,
            )


def write_deployment_doc(root: Path, body: str) -> None:
    docs = root / "docs"
    docs.mkdir()
    (docs / "deployment.md").write_text(
        "# Deployment\n\n" + body + "\n",
        encoding="utf-8",
    )


def provider_credentials_section() -> str:
    lines = ["## Provider Credentials"]
    for example in check_deployment_docs.EXPECTED_PROVIDER_EXAMPLES:
        lines.extend(
            [
                "",
                "```sh",
                "docker run --rm \\",
                f"  -e {example.env_var} \\",
                '  -v "$PWD:/work" \\',
                (
                    "  vogon-runtime:local run "
                    f"--provider {example.provider} fixtures/workflows/support-triage.toml"
                ),
                "```",
            ]
        )
    return "\n".join(lines)


if __name__ == "__main__":
    unittest.main()
