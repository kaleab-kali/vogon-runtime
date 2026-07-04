#!/usr/bin/env python3
"""Validate a Vogon run cache JSON file."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache_file", type=Path, help="Cache JSON file to validate.")
    parser.add_argument(
        "--expected-max-entries",
        type=int,
        help="Expected max_entries value.",
    )
    parser.add_argument(
        "--expected-entry-count",
        type=int,
        help="Expected number of cached outputs.",
    )
    args = parser.parse_args()

    errors = check_file(
        args.cache_file,
        expected_max_entries=args.expected_max_entries,
        expected_entry_count=args.expected_entry_count,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_file(
    path: Path,
    *,
    expected_max_entries: int | None = None,
    expected_entry_count: int | None = None,
) -> list[str]:
    try:
        output = path.read_text(encoding="utf-8")
    except OSError as error:
        return [f"cache JSON file cannot be read: {error}"]

    return check_output(
        output,
        expected_max_entries=expected_max_entries,
        expected_entry_count=expected_entry_count,
    )


def check_output(
    output: str,
    *,
    expected_max_entries: int | None = None,
    expected_entry_count: int | None = None,
) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"cache JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["cache JSON root must be an object"]

    errors: list[str] = []
    outputs = data.get("outputs")
    if not isinstance(outputs, dict):
        errors.append("cache JSON outputs must be an object")
        outputs = {}

    insertion_order = data.get("insertion_order")
    if not isinstance(insertion_order, list):
        errors.append("cache JSON insertion_order must be an array")
        insertion_order = []

    max_entries = data.get("max_entries")
    if not isinstance(max_entries, int) or max_entries < 0:
        errors.append("cache JSON max_entries must be a non-negative integer")
    elif (
        expected_max_entries is not None
        and max_entries != expected_max_entries
    ):
        errors.append(
            "cache JSON max_entries mismatch: "
            f"expected {expected_max_entries}, got {format_json_value(max_entries)}"
        )

    if expected_entry_count is not None and len(outputs) != expected_entry_count:
        errors.append(
            "cache JSON output count mismatch: "
            f"expected {expected_entry_count}, got {len(outputs)}"
        )

    if len(insertion_order) != len(outputs):
        errors.append(
            "cache JSON insertion_order length must match outputs: "
            f"expected {len(outputs)}, got {len(insertion_order)}"
        )

    for index, cache_key in enumerate(insertion_order, start=1):
        if not isinstance(cache_key, str) or not cache_key:
            errors.append(
                f"cache JSON insertion_order entry {index} must be a non-empty string"
            )
        elif cache_key not in outputs:
            errors.append(
                f"cache JSON insertion_order entry {index} is missing from outputs"
            )

    for cache_key, cached_output in outputs.items():
        if not isinstance(cache_key, str) or not cache_key:
            errors.append("cache JSON output keys must be non-empty strings")
        if not isinstance(cached_output, str) or not cached_output:
            errors.append(f"cache JSON output {cache_key} must be a non-empty string")

    return errors


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
