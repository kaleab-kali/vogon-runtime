#!/usr/bin/env python3
"""Validate machine-readable Vogon workflow check JSON output."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-workflow-name", help="Expected workflow name.")
    parser.add_argument(
        "--expected-step-count",
        type=int,
        help="Expected workflow step count.",
    )
    args = parser.parse_args()

    errors = check_output(
        sys.stdin.read(),
        expected_workflow_name=args.expected_workflow_name,
        expected_step_count=args.expected_step_count,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_output(
    output: str,
    *,
    expected_workflow_name: str | None = None,
    expected_step_count: int | None = None,
) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"workflow check JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["workflow check JSON root must be an object"]

    errors: list[str] = []
    workflow_name = data.get("workflow_name")
    if not isinstance(workflow_name, str) or not workflow_name:
        errors.append("workflow check JSON workflow_name must be a non-empty string")
    elif expected_workflow_name is not None and workflow_name != expected_workflow_name:
        errors.append(
            "workflow check JSON workflow_name mismatch: "
            f"expected {expected_workflow_name}, got {format_json_value(workflow_name)}"
        )

    step_count = data.get("step_count")
    if not isinstance(step_count, int) or step_count < 1:
        errors.append("workflow check JSON step_count must be a positive integer")
    elif expected_step_count is not None and step_count != expected_step_count:
        errors.append(
            "workflow check JSON step_count mismatch: "
            f"expected {expected_step_count}, got {format_json_value(step_count)}"
        )

    return errors


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
