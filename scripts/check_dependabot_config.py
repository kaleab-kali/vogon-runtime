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
        "groups.cargo-minor-patch.patterns": "*",
        "groups.cargo-minor-patch.update-types": "minor,patch",
        "commit-message.prefix": "deps",
    },
    "github-actions": {
        "directory": "/",
        "interval": "weekly",
        "open-pull-requests-limit": "5",
        "groups.github-actions-minor-patch.patterns": "*",
        "groups.github-actions-minor-patch.update-types": "minor,patch",
        "commit-message.prefix": "ci",
    },
    "docker": {
        "directory": "/",
        "interval": "weekly",
        "open-pull-requests-limit": "5",
        "groups.docker-minor-patch.patterns": "*",
        "groups.docker-minor-patch.update-types": "minor,patch",
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
    in_groups = False
    current_group: str | None = None
    in_group_patterns = False
    in_group_update_types = False

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
            in_groups = False
            current_group = None
            in_group_patterns = False
            in_group_update_types = False
            continue

        if current_ecosystem is None:
            continue

        if stripped == "schedule:":
            in_schedule = True
            in_commit_message = False
            in_groups = False
            continue
        if stripped == "commit-message:":
            in_commit_message = True
            in_schedule = False
            in_groups = False
            current_group = None
            continue
        if stripped == "groups:":
            in_groups = True
            in_schedule = False
            in_commit_message = False
            current_group = None
            continue
        if (
            in_groups
            and stripped.endswith(":")
            and stripped not in {"patterns:", "update-types:"}
        ):
            current_group = stripped.removesuffix(":")
            in_group_patterns = False
            in_group_update_types = False
            continue
        if in_groups and current_group is not None and stripped == "patterns:":
            in_group_patterns = True
            in_group_update_types = False
            updates[current_ecosystem][f"groups.{current_group}.patterns"] = ""
            continue
        if in_groups and current_group is not None and stripped == "update-types:":
            in_group_patterns = False
            in_group_update_types = True
            updates[current_ecosystem][f"groups.{current_group}.update-types"] = ""
            continue
        if in_groups and current_group is not None and stripped.startswith("- "):
            value = stripped.removeprefix("- ").strip().strip('"')
            key = (
                f"groups.{current_group}.patterns"
                if in_group_patterns
                else f"groups.{current_group}.update-types"
            )
            existing = updates[current_ecosystem].get(key)
            updates[current_ecosystem][key] = (
                value if not existing else f"{existing},{value}"
            )
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
