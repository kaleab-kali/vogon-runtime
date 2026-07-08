#!/usr/bin/env python3
"""Validate changelog structure for release readiness."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


ALLOWED_UNRELEASED_SECTIONS = {
    "Added",
    "Changed",
    "Deprecated",
    "Removed",
    "Fixed",
    "Security",
    "Documentation",
}
RELEASE_HEADING_PREFIX = "## ["


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
    changelog = root / "CHANGELOG.md"
    if not changelog.exists():
        return ["CHANGELOG.md: missing changelog"]

    lines = changelog.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []

    if not lines or lines[0] != "# Changelog":
        errors.append("CHANGELOG.md: first line must be `# Changelog`")

    text = "\n".join(lines)
    if "https://keepachangelog.com/en/1.1.0/" not in text:
        errors.append("CHANGELOG.md: missing Keep a Changelog 1.1.0 reference")
    if "semantic versioning" not in text.lower():
        errors.append("CHANGELOG.md: missing semantic versioning note")

    try:
        unreleased_start = lines.index("## [Unreleased]")
    except ValueError:
        errors.append("CHANGELOG.md: missing `## [Unreleased]` section")
        return errors

    next_heading = next_release_heading(lines, unreleased_start + 1)
    unreleased_lines = lines[unreleased_start + 1 : next_heading]
    errors.extend(check_unreleased_section(unreleased_lines, has_release=next_heading < len(lines)))
    errors.extend(check_release_headings(lines[next_heading:]))
    return errors


def next_release_heading(lines: list[str], start: int) -> int:
    for index, line in enumerate(lines[start:], start=start):
        if line.startswith("## ") and line != "## [Unreleased]":
            return index
    return len(lines)


def check_unreleased_section(lines: list[str], *, has_release: bool) -> list[str]:
    errors: list[str] = []
    section_names = [line[4:] for line in lines if line.startswith("### ")]
    if not section_names:
        if has_release and not any(line.strip() for line in lines):
            return []
        return ["CHANGELOG.md: `## [Unreleased]` must contain at least one subsection"]

    for section_name in section_names:
        if section_name not in ALLOWED_UNRELEASED_SECTIONS:
            errors.append(f"CHANGELOG.md: unsupported Unreleased subsection `{section_name}`")

    for section_name in section_names:
        if not section_has_entry(lines, section_name):
            errors.append(f"CHANGELOG.md: Unreleased `{section_name}` subsection has no entries")

    return errors


def check_release_headings(lines: list[str]) -> list[str]:
    errors: list[str] = []
    for line in lines:
        if line.startswith("## "):
            if not line.startswith(RELEASE_HEADING_PREFIX) or " - " not in line:
                errors.append(
                    f"CHANGELOG.md: release heading `{line}` must include a version and date"
                )
    return errors


def section_has_entry(lines: list[str], section_name: str) -> bool:
    heading = f"### {section_name}"
    in_section = False
    for line in lines:
        if line == heading:
            in_section = True
            continue
        if in_section and line.startswith("### "):
            return False
        if in_section and line.startswith("- "):
            return True
    return False


if __name__ == "__main__":
    raise SystemExit(main())
