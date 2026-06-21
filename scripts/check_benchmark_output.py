#!/usr/bin/env python3
"""Validate Vogon benchmark smoke output."""

from __future__ import annotations

import argparse
import math
import sys


REQUIRED_METRICS = {
    "iterations",
    "elapsed_ms",
    "iterations_per_second",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--expected-iterations",
        type=int,
        required=True,
        help="Expected benchmark iteration count.",
    )
    args = parser.parse_args()

    output = sys.stdin.read()
    errors = check_output(output, expected_iterations=args.expected_iterations)
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_output(output: str, *, expected_iterations: int) -> list[str]:
    metrics = parse_metrics(output)
    errors: list[str] = []

    missing_metrics = sorted(REQUIRED_METRICS.difference(metrics))
    for metric in missing_metrics:
        errors.append(f"missing benchmark metric: {metric}")

    iterations = parse_int_metric(metrics, "iterations", errors)
    if iterations is not None and iterations != expected_iterations:
        errors.append(
            f"benchmark iterations mismatch: expected {expected_iterations}, got {iterations}"
        )

    elapsed_ms = parse_float_metric(metrics, "elapsed_ms", errors)
    if elapsed_ms is not None and elapsed_ms <= 0:
        errors.append("benchmark elapsed_ms must be greater than zero")

    iterations_per_second = parse_float_metric(metrics, "iterations_per_second", errors)
    if iterations_per_second is not None and iterations_per_second <= 0:
        errors.append("benchmark iterations_per_second must be greater than zero")

    return errors


def parse_metrics(output: str) -> dict[str, str]:
    metrics: dict[str, str] = {}
    for line in output.splitlines():
        name, separator, value = line.partition(":")
        if not separator:
            continue
        normalized_name = name.strip()
        if normalized_name in REQUIRED_METRICS:
            metrics[normalized_name] = value.strip()
    return metrics


def parse_int_metric(
    metrics: dict[str, str], name: str, errors: list[str]
) -> int | None:
    if name not in metrics:
        return None
    try:
        value = int(metrics[name])
    except ValueError:
        errors.append(f"benchmark {name} must be an integer")
        return None
    if value <= 0:
        errors.append(f"benchmark {name} must be greater than zero")
    return value


def parse_float_metric(
    metrics: dict[str, str], name: str, errors: list[str]
) -> float | None:
    if name not in metrics:
        return None
    try:
        value = float(metrics[name])
    except ValueError:
        errors.append(f"benchmark {name} must be a number")
        return None
    if not math.isfinite(value):
        errors.append(f"benchmark {name} must be finite")
        return None
    return value


if __name__ == "__main__":
    sys.exit(main())
