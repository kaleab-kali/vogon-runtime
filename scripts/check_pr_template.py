#!/usr/bin/env python3
"""Validate that the pull request template includes README local checks."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


README_MARKER = "Run local checks:"


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
    pr_template = root / ".github" / "pull_request_template.md"
    if not readme.exists():
        return ["README.md: missing README local checks"]
    if not pr_template.exists():
        return [".github/pull_request_template.md: missing pull request template"]

    readme_commands = extract_shell_commands(readme, README_MARKER)
    template_commands = extract_template_commands(pr_template)
    errors: list[str] = []

    if not readme_commands:
        errors.append("README.md: missing local check command block")
    if not template_commands:
        errors.append(".github/pull_request_template.md: missing verification command checklist")

    template_command_set = set(template_commands)
    for command in readme_commands:
        if command not in template_command_set:
            errors.append(
                f".github/pull_request_template.md: missing README local check `{command}`"
            )

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


def extract_template_commands(path: Path) -> list[str]:
    commands: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        prefix = "- [ ] `"
        if stripped.startswith(prefix) and stripped.endswith("`"):
            commands.append(stripped[len(prefix) : -1])
    return commands


if __name__ == "__main__":
    raise SystemExit(main())
