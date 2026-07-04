import tempfile
import unittest
from pathlib import Path

from scripts import check_live_workflows
from scripts.check_live_replay import PROVIDERS


class CheckLiveWorkflowsTests(unittest.TestCase):
    def test_accepts_current_live_workflow_contract(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_all_live_workflows(root)

            self.assertEqual(check_live_workflows.check_repository(root), [])

    def test_reports_missing_expected_workflow(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_all_live_workflows(root, skip_provider="groq")

            errors = check_live_workflows.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/workflows/live-groq-smoke.yml: missing live provider smoke workflow"
                ],
            )

    def test_reports_unexpected_live_workflow(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_all_live_workflows(root)
            workflows = root / ".github" / "workflows"
            (workflows / "live-extra-smoke.yml").write_text(
                "name: Extra\n", encoding="utf-8"
            )

            errors = check_live_workflows.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/workflows/live-extra-smoke.yml: unexpected live provider smoke workflow"
                ],
            )

    def test_reports_missing_replay_validator(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_all_live_workflows(root, omit_live_validator_for="openrouter")

            errors = check_live_workflows.check_repository(root)

            self.assertIn(
                ".github/workflows/live-openrouter-smoke.yml: missing live replay validator",
                errors,
            )

    def test_reports_wrong_secret_wiring(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_all_live_workflows(root)
            workflow = root / ".github" / "workflows" / "live-gemini-smoke.yml"
            workflow.write_text(
                workflow.read_text(encoding="utf-8").replace(
                    "--secret-env GEMINI_API_KEY",
                    "--secret-env WRONG_SECRET",
                ),
                encoding="utf-8",
            )

            errors = check_live_workflows.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/workflows/live-gemini-smoke.yml: missing validator secret env"
                ],
            )


def write_all_live_workflows(
    root: Path,
    *,
    skip_provider: str | None = None,
    omit_live_validator_for: str | None = None,
) -> None:
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    for expectation in check_live_workflows.EXPECTED_WORKFLOWS.values():
        if expectation.provider == skip_provider:
            continue
        workflow = workflows / expectation.file_name
        workflow.write_text(
            live_workflow_text(
                expectation,
                omit_live_validator=expectation.provider == omit_live_validator_for,
            ),
            encoding="utf-8",
        )


def live_workflow_text(
    expectation: check_live_workflows.LiveWorkflowExpectation,
    *,
    omit_live_validator: bool = False,
) -> str:
    provider = PROVIDERS[expectation.provider]
    base_url_run_flag = ""
    base_url_validator_flag = ""
    if expectation.requires_base_url:
        base_url_run_flag = (
            f"\n            --{expectation.flag_prefix}-base-url \"$base_url\" \\"
        )
        base_url_validator_flag = "\n            --base-url \"$base_url\" \\"

    validator = ""
    if not omit_live_validator:
        validator = f"""
          python3 scripts/check_live_replay.py \\
            --replay {expectation.replay_path} \\
            --provider {expectation.provider} \\{base_url_validator_flag}
            --model "$model" \\
            --secret-env {provider.secret_env}"""

    return f"""name: Live {expectation.provider} Smoke

on:
  workflow_dispatch:
  workflow_call:
    secrets:
      {provider.secret_env}:
        required: true

permissions:
  contents: read

jobs:
  live:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    env:
      {provider.secret_env}: ${{{{ secrets.{provider.secret_env} }}}}

    steps:
      - name: Checkout
        uses: actions/checkout@v7

      - name: Show Rust toolchain
        run: rustup show

      - name: Build CLI
        run: cargo build --release -p vogon-cli --locked

      - name: Run workflow smoke
        run: |
          if [ -z "${{{provider.secret_env}:-}}" ]; then
            exit 1
          fi

          model="test-model"
          ./target/release/vogon run \\
            --provider {expectation.provider}{base_url_run_flag}
            --{expectation.flag_prefix}-model "$model" \\
            --{expectation.flag_prefix}-timeout-seconds 60 \\
            --{expectation.flag_prefix}-max-retries 2 \\
            --redact {provider.redaction_label}="${provider.secret_env}" \\
            --output {expectation.replay_path} \\
            fixtures/workflows/support-triage.toml
{validator}
"""


if __name__ == "__main__":
    unittest.main()
