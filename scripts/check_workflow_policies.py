#!/usr/bin/env python3
"""Validate GitHub Actions workflow security policy conventions."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


WORKFLOW_SUFFIXES = {".yml", ".yaml"}
BROAD_PERMISSION_RE = re.compile(r"^\s*permissions:\s*(?:read-all|write-all)\s*$")
PERMISSION_VALUE_RE = re.compile(r"^\s+([A-Za-z0-9_-]+):\s*([A-Za-z-]+)\s*$")
TOP_LEVEL_KEY_RE = re.compile(r"^[A-Za-z0-9_-]+:")
ALLOWED_TOP_LEVEL_WRITE_SCOPES = {"security-events"}


@dataclass(frozen=True)
class TopLevelPermissions:
    line: int
    entries: dict[str, tuple[str, int]]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root to scan. Defaults to the current directory.",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    errors = check_repository(root)
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_repository(root: Path) -> list[str]:
    errors: list[str] = []
    for workflow_file in workflow_files(root):
        errors.extend(check_workflow_file(root, workflow_file))
    return errors


def workflow_files(root: Path) -> list[Path]:
    workflows_dir = root / ".github" / "workflows"
    if not workflows_dir.exists():
        return []
    return sorted(
        path
        for path in workflows_dir.iterdir()
        if path.is_file() and path.suffix.lower() in WORKFLOW_SUFFIXES
    )


def check_workflow_file(root: Path, path: Path) -> list[str]:
    relative_path = path.relative_to(root).as_posix()
    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []

    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("pull_request_target:"):
            errors.append(
                f"{relative_path}:{line_number}: pull_request_target is not allowed"
            )
        if BROAD_PERMISSION_RE.match(line):
            errors.append(
                f"{relative_path}:{line_number}: broad workflow permissions are not allowed"
            )

    permissions = parse_top_level_permissions(lines)
    if permissions is None:
        errors.append(f"{relative_path}: missing top-level permissions block")
        return errors

    jobs_line = first_top_level_key_line(lines, "jobs:")
    if jobs_line is not None and permissions.line > jobs_line:
        errors.append(
            f"{relative_path}:{permissions.line}: top-level permissions must be before jobs"
        )

    contents = permissions.entries.get("contents")
    if contents is None:
        errors.append(
            f"{relative_path}:{permissions.line}: top-level permissions must include contents"
        )
    elif contents[0] != "read":
        errors.append(
            f"{relative_path}:{contents[1]}: top-level contents permission must be read"
        )

    for scope, (level, line_number) in permissions.entries.items():
        if level == "write" and scope not in ALLOWED_TOP_LEVEL_WRITE_SCOPES:
            errors.append(
                f"{relative_path}:{line_number}: top-level {scope} write permission "
                "must be job-scoped"
            )

    return errors


def parse_top_level_permissions(lines: list[str]) -> TopLevelPermissions | None:
    for index, line in enumerate(lines):
        if line == "permissions:":
            entries: dict[str, tuple[str, int]] = {}
            for child_index in range(index + 1, len(lines)):
                child = lines[child_index]
                if is_top_level_key(child):
                    break
                match = PERMISSION_VALUE_RE.match(child)
                if match:
                    scope, level = match.groups()
                    entries[scope] = (level, child_index + 1)
            return TopLevelPermissions(line=index + 1, entries=entries)
    return None


def first_top_level_key_line(lines: list[str], key: str) -> int | None:
    for line_number, line in enumerate(lines, start=1):
        if line == key:
            return line_number
    return None


def is_top_level_key(line: str) -> bool:
    return (
        bool(line)
        and not line.startswith((" ", "\t"))
        and bool(TOP_LEVEL_KEY_RE.match(line))
    )


if __name__ == "__main__":
    sys.exit(main())
