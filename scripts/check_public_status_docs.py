#!/usr/bin/env python3
"""Validate public project status wording after the first release."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


REQUIRED_SNIPPETS = {
    "README.md": [
        "Vogon Runtime has a first public release, `v0.1.0`.",
        "The project is still in\n"
        "the `0.x` series, so command and library APIs may change",
    ],
    "SECURITY.md": [
        "`v0.1.0` is the first public release of Vogon Runtime.",
        "shipped in follow-up patch or minor releases",
    ],
    "SUPPORT.md": [
        "Vogon Runtime is released open-source software in the `0.x` series.",
    ],
    "CHANGELOG.md": [
        "and this project follows semantic versioning.",
        "## [0.1.0] - 2026-07-08",
    ],
    "docs/release.md": [
        "still in the `0.x` series",
    ],
}

STALE_PHRASES = [
    "Vogon Runtime is pre-release",
    "has not published a stable release yet",
    "until `v0.1.0` is tagged",
    "once the first release is tagged",
    "public API is\npre-release",
]


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
    errors: list[str] = []
    for relative_path, snippets in REQUIRED_SNIPPETS.items():
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path}: missing public status document")
            continue

        text = path.read_text(encoding="utf-8")
        for snippet in snippets:
            if snippet not in text:
                errors.append(f'{relative_path}: missing "{single_line(snippet)}"')
        for phrase in STALE_PHRASES:
            if phrase in text:
                errors.append(
                    f'{relative_path}: stale status phrase "{single_line(phrase)}"'
                )

    return errors


def single_line(text: str) -> str:
    return " ".join(text.split())


if __name__ == "__main__":
    raise SystemExit(main())
