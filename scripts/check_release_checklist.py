#!/usr/bin/env python3
"""Validate that release verification docs include README local checks."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


README_MARKER = "Run local checks:"
RELEASE_MARKER = "Run the full local verification set:"


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
    release_doc = root / "docs" / "release.md"
    if not readme.exists():
        return ["README.md: missing README local checks"]
    if not release_doc.exists():
        return ["docs/release.md: missing release process documentation"]

    readme_commands = extract_shell_commands(readme, README_MARKER)
    release_commands = extract_shell_commands(release_doc, RELEASE_MARKER)
    errors: list[str] = []

    if not readme_commands:
        errors.append("README.md: missing local check command block")
    if not release_commands:
        errors.append("docs/release.md: missing release verification command block")

    release_command_set = set(release_commands)
    for command in readme_commands:
        if command not in release_command_set:
            errors.append(f"docs/release.md: missing README local check `{command}`")

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
