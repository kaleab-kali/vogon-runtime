#!/usr/bin/env python3
"""Validate Dependabot coverage for maintained dependency surfaces."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


EXPECTED_UPDATES = {
    "cargo": {
        "directory": "/",
        "interval": "weekly",
        "open-pull-requests-limit": "5",
        "commit-message.prefix": "deps",
    },
    "github-actions": {
        "directory": "/",
        "interval": "weekly",
        "open-pull-requests-limit": "5",
        "commit-message.prefix": "ci",
    },
    "docker": {
        "directory": "/",
        "interval": "weekly",
        "open-pull-requests-limit": "5",
        "commit-message.prefix": "deps",
    },
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
    path = root / ".github" / "dependabot.yml"
    if not path.exists():
        return [".github/dependabot.yml: missing Dependabot configuration"]

    text = path.read_text(encoding="utf-8")
    updates = parse_update_blocks(text)
    errors: list[str] = []

    if not text.startswith("version: 2\n"):
        errors.append(".github/dependabot.yml: missing version 2 declaration")

    for ecosystem, expected in EXPECTED_UPDATES.items():
        config = updates.get(ecosystem)
        if config is None:
            errors.append(f".github/dependabot.yml: missing {ecosystem} updates")
            continue

        for key, value in expected.items():
            if config.get(key) != value:
                errors.append(
                    f".github/dependabot.yml: {ecosystem} `{key}` must be {value!r}"
                )

    return errors


def parse_update_blocks(text: str) -> dict[str, dict[str, str]]:
    updates: dict[str, dict[str, str]] = {}
    current_ecosystem: str | None = None
    in_schedule = False
    in_commit_message = False

    for raw_line in text.splitlines():
        line = raw_line.rstrip()
        stripped = line.strip()
        if not stripped:
            continue

        if stripped.startswith("- package-ecosystem:"):
            current_ecosystem = stripped.split(":", 1)[1].strip()
            updates[current_ecosystem] = {}
            in_schedule = False
            in_commit_message = False
            continue

        if current_ecosystem is None:
            continue

        if stripped == "schedule:":
            in_schedule = True
            in_commit_message = False
            continue
        if stripped == "commit-message:":
            in_commit_message = True
            in_schedule = False
            continue
        if stripped.endswith(":") and stripped not in {"schedule:", "commit-message:"}:
            in_schedule = False
            in_commit_message = False

        if ":" not in stripped:
            continue

        key, value = [part.strip() for part in stripped.split(":", 1)]
        if key == "directory":
            updates[current_ecosystem]["directory"] = value
        elif key == "open-pull-requests-limit":
            updates[current_ecosystem]["open-pull-requests-limit"] = value
        elif in_schedule and key == "interval":
            updates[current_ecosystem]["interval"] = value
        elif in_commit_message and key == "prefix":
            updates[current_ecosystem]["commit-message.prefix"] = value

    return updates


if __name__ == "__main__":
    raise SystemExit(main())
