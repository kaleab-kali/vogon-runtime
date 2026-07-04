#!/usr/bin/env python3
"""Validate live provider smoke workflow wiring."""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path

try:
    from scripts.check_live_replay import PROVIDERS
except ModuleNotFoundError:
    from check_live_replay import PROVIDERS


WORKFLOW_PATTERN = "live-*-smoke.y*ml"


@dataclass(frozen=True)
class LiveWorkflowExpectation:
    provider: str
    file_name: str
    flag_prefix: str
    replay_path: str
    requires_base_url: bool = False


EXPECTED_WORKFLOWS = {
    "gemini": LiveWorkflowExpectation(
        provider="gemini",
        file_name="live-gemini-smoke.yml",
        flag_prefix="gemini",
        replay_path="target/live-gemini-smoke.replay.json",
    ),
    "groq": LiveWorkflowExpectation(
        provider="groq",
        file_name="live-groq-smoke.yml",
        flag_prefix="groq",
        replay_path="target/live-groq-smoke.replay.json",
    ),
    "hugging-face": LiveWorkflowExpectation(
        provider="hugging-face",
        file_name="live-hugging-face-smoke.yml",
        flag_prefix="hugging-face",
        replay_path="target/live-hugging-face-smoke.replay.json",
    ),
    "openai-compatible": LiveWorkflowExpectation(
        provider="openai-compatible",
        file_name="live-openai-compatible-smoke.yml",
        flag_prefix="openai-compatible",
        replay_path="target/live-openai-compatible-smoke.replay.json",
        requires_base_url=True,
    ),
    "openrouter": LiveWorkflowExpectation(
        provider="openrouter",
        file_name="live-openrouter-smoke.yml",
        flag_prefix="openrouter",
        replay_path="target/live-openrouter-smoke.replay.json",
    ),
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root to scan. Defaults to the current directory.",
    )
    args = parser.parse_args()

    errors = check_repository(args.root.resolve())
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_repository(root: Path) -> list[str]:
    workflows_dir = root / ".github" / "workflows"
    errors: list[str] = []

    expected_files = {expectation.file_name for expectation in EXPECTED_WORKFLOWS.values()}
    actual_files = {
        path.name
        for path in workflows_dir.glob(WORKFLOW_PATTERN)
        if path.is_file() and path.suffix.lower() in {".yml", ".yaml"}
    }

    for missing in sorted(expected_files - actual_files):
        errors.append(f".github/workflows/{missing}: missing live provider smoke workflow")

    for unexpected in sorted(actual_files - expected_files):
        errors.append(
            f".github/workflows/{unexpected}: unexpected live provider smoke workflow"
        )

    for expectation in EXPECTED_WORKFLOWS.values():
        path = workflows_dir / expectation.file_name
        if path.exists():
            errors.extend(check_workflow_file(root, path, expectation))

    return errors


def check_workflow_file(
    root: Path,
    path: Path,
    expectation: LiveWorkflowExpectation,
) -> list[str]:
    relative_path = path.relative_to(root).as_posix()
    text = path.read_text(encoding="utf-8")
    provider = PROVIDERS[expectation.provider]
    secret_ref = "${{ secrets." + provider.secret_env + " }}"

    required_snippets = {
        "workflow_dispatch trigger": "  workflow_dispatch:",
        "workflow_call trigger": "  workflow_call:",
        "read-only top-level contents permission": "permissions:\n  contents: read",
        "job timeout": "    timeout-minutes:",
        "workflow_call secret declaration": (
            f"      {provider.secret_env}:\n        required: true"
        ),
        "checkout step": "        uses: actions/checkout@v7",
        "Rust toolchain step": "        run: rustup show",
        "release CLI build": "        run: cargo build --release -p vogon-cli --locked",
        "secret env wiring": f"      {provider.secret_env}: {secret_ref}",
        "secret presence guard": f'if [ -z "${{{provider.secret_env}:-}}" ]; then',
        "provider run flag": f"            --provider {expectation.provider}",
        "timeout run flag": (
            f"            --{expectation.flag_prefix}-timeout-seconds 60"
        ),
        "retry run flag": f"            --{expectation.flag_prefix}-max-retries 2",
        "redaction run flag": (
            f'            --redact {provider.redaction_label}="$'
            f'{provider.secret_env}"'
        ),
        "replay output path": f"            --output {expectation.replay_path}",
        "live replay validator": "          python3 scripts/check_live_replay.py",
        "validator replay path": f"            --replay {expectation.replay_path}",
        "validator provider": f"            --provider {expectation.provider}",
        "validator model": "            --model",
        "validator secret env": f"            --secret-env {provider.secret_env}",
    }

    if expectation.requires_base_url:
        required_snippets["base URL run flag"] = (
            f"            --{expectation.flag_prefix}-base-url"
        )
        required_snippets["validator base URL"] = "            --base-url"

    errors = []
    for description, snippet in required_snippets.items():
        if snippet not in text:
            errors.append(f"{relative_path}: missing {description}")

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
