#!/usr/bin/env python3
"""Validate required files in an extracted release archive."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


DEFAULT_REQUIRED_FILES = ["README.md", "LICENSE"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "archive_directory",
        type=Path,
        help="Directory containing extracted release archive contents.",
    )
    parser.add_argument(
        "--binary",
        required=True,
        help="Expected CLI binary filename inside the archive directory.",
    )
    parser.add_argument(
        "--required-file",
        action="append",
        default=[],
        help=(
            "Required non-binary file inside the archive directory. "
            "May be repeated. Defaults to README.md and LICENSE."
        ),
    )
    args = parser.parse_args()

    errors = check_directory(
        args.archive_directory,
        binary=args.binary,
        required_files=args.required_file or DEFAULT_REQUIRED_FILES,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_directory(
    archive_directory: Path,
    *,
    binary: str,
    required_files: list[str] | None = None,
) -> list[str]:
    if not archive_directory.is_dir():
        return [f"Archive directory is missing or is not a directory: {archive_directory}"]

    errors: list[str] = []
    required_files = required_files or DEFAULT_REQUIRED_FILES

    binary_path = archive_directory / binary
    if not binary_path.exists():
        errors.append(f"Packaged archive binary is missing: {binary}")
    elif not binary_path.is_file():
        errors.append(f"Packaged archive binary is not a regular file: {binary}")

    for required_file in required_files:
        path = archive_directory / required_file
        if not path.exists():
            errors.append(f"Packaged archive required file is missing: {required_file}")
        elif not path.is_file():
            errors.append(
                f"Packaged archive required file is not a regular file: {required_file}"
            )

    return errors


if __name__ == "__main__":
    sys.exit(main())
