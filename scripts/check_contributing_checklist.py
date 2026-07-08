#!/usr/bin/env python3
"""Validate that contributor docs include README local checks and live workflow guidance."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


README_MARKER = "Run local checks:"
CONTRIBUTING_MARKER = "## Development"
LIVE_WORKFLOW_GUIDANCE = {
    "Live Gemini Smoke": "GEMINI_API_KEY",
    "Live Groq Smoke": "GROQ_API_KEY",
    "Live Hugging Face Smoke": "HF_TOKEN",
    "Live OpenAI-Compatible Smoke": "OPENAI_COMPATIBLE_API_KEY",
    "Live OpenRouter Smoke": "OPENROUTER_API_KEY",
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
    readme = root / "README.md"
    contributing = root / "CONTRIBUTING.md"
    if not readme.exists():
        return ["README.md: missing README local checks"]
    if not contributing.exists():
        return ["CONTRIBUTING.md: missing contributor documentation"]

    readme_commands = extract_shell_commands(readme, README_MARKER)
    contributing_commands = extract_shell_commands(contributing, CONTRIBUTING_MARKER)
    errors: list[str] = []

    if not readme_commands:
        errors.append("README.md: missing local check command block")
    if not contributing_commands:
        errors.append("CONTRIBUTING.md: missing development command block")

    contributing_command_set = set(contributing_commands)
    for command in readme_commands:
        if command not in contributing_command_set:
            errors.append(f"CONTRIBUTING.md: missing README local check `{command}`")

    contributing_text = contributing.read_text(encoding="utf-8")
    for workflow_name, secret_name in LIVE_WORKFLOW_GUIDANCE.items():
        if workflow_name not in contributing_text:
            errors.append(f"CONTRIBUTING.md: missing `{workflow_name}` guidance")
        if secret_name not in contributing_text:
            errors.append(f"CONTRIBUTING.md: missing `{secret_name}` live smoke secret guidance")

    return errors


def extract_shell_commands(path: Path, marker: str) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    try:
        marker_index = lines.index(marker)
    except ValueError:
        return []

    in_block = False
    commands: list[str] = []
    for line in lines[marker_index + 1 :]:
        stripped = line.strip()
        if stripped.startswith("```"):
            if in_block:
                return commands
            in_block = stripped in {"```sh", "```shell", "```bash"}
            continue
        if in_block and stripped:
            commands.append(stripped)

    return commands


if __name__ == "__main__":
    raise SystemExit(main())
