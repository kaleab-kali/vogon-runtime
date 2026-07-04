#!/usr/bin/env python3
"""Validate machine-readable Vogon trace JSON Lines output."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-provider", help="Expected runtime provider.")
    parser.add_argument("--expected-model", help="Expected runtime model.")
    parser.add_argument(
        "--expected-schema-version",
        default=1,
        type=int,
        help="Expected replay schema version. Defaults to 1.",
    )
    parser.add_argument(
        "--expected-step-count",
        type=int,
        help="Expected number of step events.",
    )
    args = parser.parse_args()

    errors = check_output(
        sys.stdin.read(),
        expected_provider=args.expected_provider,
        expected_model=args.expected_model,
        expected_schema_version=args.expected_schema_version,
        expected_step_count=args.expected_step_count,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_output(
    output: str,
    *,
    expected_provider: str | None = None,
    expected_model: str | None = None,
    expected_schema_version: int = 1,
    expected_step_count: int | None = None,
) -> list[str]:
    lines = [line for line in output.splitlines() if line.strip()]
    if not lines:
        return ["trace JSONL output must not be empty"]

    events: list[Any] = []
    errors: list[str] = []
    for index, line in enumerate(lines, start=1):
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            errors.append(f"trace JSONL line {index} is invalid JSON: {error}")
            continue
        if not isinstance(event, dict):
            errors.append(f"trace JSONL line {index} must be an object")
            continue
        events.append(event)

    if errors:
        return errors

    run = events[0]
    if run.get("event") != "run":
        errors.append("trace JSONL first event must be run")

    if run.get("schema_version") != expected_schema_version:
        errors.append(
            "trace JSONL schema_version mismatch: "
            f"expected {expected_schema_version}, got {format_json_value(run.get('schema_version'))}"
        )

    runtime = run.get("runtime")
    if not isinstance(runtime, dict):
        errors.append("trace JSONL run runtime must be an object")
        runtime = {}

    if expected_provider is not None and runtime.get("provider") != expected_provider:
        errors.append(
            "trace JSONL runtime provider mismatch: "
            f"expected {expected_provider}, got {format_json_value(runtime.get('provider'))}"
        )

    if expected_model is not None and runtime.get("model") != expected_model:
        errors.append(
            "trace JSONL runtime model mismatch: "
            f"expected {expected_model}, got {format_json_value(runtime.get('model'))}"
        )

    step_count = run.get("step_count")
    if not isinstance(step_count, int) or step_count < 1:
        errors.append("trace JSONL run step_count must be a positive integer")

    step_events = events[1:]
    if expected_step_count is not None and len(step_events) != expected_step_count:
        errors.append(
            "trace JSONL step event count mismatch: "
            f"expected {expected_step_count}, got {len(step_events)}"
        )
    if isinstance(step_count, int) and step_count != len(step_events):
        errors.append(
            "trace JSONL run step_count must match step events: "
            f"expected {step_count}, got {len(step_events)}"
        )

    for expected_index, step in enumerate(step_events, start=1):
        errors.extend(check_step_event(step, expected_index))

    return errors


def check_step_event(step: dict[str, Any], expected_index: int) -> list[str]:
    errors: list[str] = []
    if step.get("event") != "step":
        errors.append(f"trace JSONL event {expected_index + 1} must be step")
    if step.get("index") != expected_index:
        errors.append(
            f"trace JSONL step index mismatch at event {expected_index + 1}: "
            f"expected {expected_index}, got {format_json_value(step.get('index'))}"
        )

    for field in ("step_id", "input_hash", "output_hash", "output"):
        if not isinstance(step.get(field), str) or not step[field]:
            errors.append(
                f"trace JSONL step {expected_index} field {field} must be a non-empty string"
            )

    return errors


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
