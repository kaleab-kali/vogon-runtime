#!/usr/bin/env python3
"""Validate machine-readable Vogon replay verification JSON output."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-workflow-name", help="Expected workflow name.")
    match_group = parser.add_mutually_exclusive_group()
    match_group.add_argument(
        "--expect-match",
        action="store_true",
        help="Require is_match=true and an empty mismatch list.",
    )
    match_group.add_argument(
        "--expect-mismatch",
        action="store_true",
        help="Require is_match=false and a non-empty mismatch list.",
    )
    args = parser.parse_args()

    expected_match = None
    if args.expect_match:
        expected_match = True
    elif args.expect_mismatch:
        expected_match = False

    errors = check_output(
        sys.stdin.read(),
        expected_workflow_name=args.expected_workflow_name,
        expected_match=expected_match,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_output(
    output: str,
    *,
    expected_workflow_name: str | None = None,
    expected_match: bool | None = None,
) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"verify JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["verify JSON root must be an object"]

    errors: list[str] = []
    workflow_name = data.get("workflow_name")
    if not isinstance(workflow_name, str) or not workflow_name:
        errors.append("verify JSON workflow_name must be a non-empty string")
    elif expected_workflow_name is not None and workflow_name != expected_workflow_name:
        errors.append(
            "verify JSON workflow_name mismatch: "
            f"expected {expected_workflow_name}, got {format_json_value(workflow_name)}"
        )

    is_match = data.get("is_match")
    if not isinstance(is_match, bool):
        errors.append("verify JSON is_match must be a boolean")
    elif expected_match is not None and is_match is not expected_match:
        errors.append(
            "verify JSON is_match mismatch: "
            f"expected {format_json_value(expected_match)}, got {format_json_value(is_match)}"
        )

    mismatches = data.get("mismatches")
    if not isinstance(mismatches, list):
        errors.append("verify JSON mismatches must be an array")
    elif is_match is True and mismatches:
        errors.append("verify JSON mismatches must be empty when is_match is true")
    elif expected_match is True and mismatches:
        errors.append("verify JSON mismatches must be empty for expected matches")
    elif expected_match is False and not mismatches:
        errors.append("verify JSON mismatches must be non-empty for expected mismatches")

    return errors


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
