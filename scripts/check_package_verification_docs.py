#!/usr/bin/env python3
"""Validate package verification documentation for release readiness."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


PACKAGE_COMMAND = "cargo package --workspace --allow-dirty --no-verify --offline"
RATIONALE_SNIPPETS = [
    "Cargo can fail offline verification while resolving unpublished internal workspace crates",
    "preceding build, test, docs, install, and smoke commands",
]
DOCUMENTED_PATHS = ["README.md", "docs/release.md"]


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
    for relative_path in DOCUMENTED_PATHS:
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path}: missing package verification documentation")
            continue

        text = path.read_text(encoding="utf-8")
        normalized_text = normalize_whitespace(text)
        if PACKAGE_COMMAND not in text:
            errors.append(f"{relative_path}: missing offline package command")
        for snippet in RATIONALE_SNIPPETS:
            if snippet not in normalized_text:
                errors.append(f"{relative_path}: missing package verification rationale")
                break

    return errors


def normalize_whitespace(text: str) -> str:
    return re.sub(r"\s+", " ", text)


if __name__ == "__main__":
    raise SystemExit(main())
