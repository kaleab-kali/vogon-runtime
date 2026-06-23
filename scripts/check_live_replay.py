#!/usr/bin/env python3
"""Validate replay shape from a live provider smoke workflow."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ProviderExpectation:
    provider: str
    adapter: str
    base_url: str | None
    secret_env: str
    redaction_label: str


PROVIDERS = {
    "gemini": ProviderExpectation(
        provider="gemini",
        adapter="gemini-generate-content",
        base_url="https://generativelanguage.googleapis.com",
        secret_env="GEMINI_API_KEY",
        redaction_label="gemini_api_key",
    ),
    "groq": ProviderExpectation(
        provider="groq",
        adapter="groq-openai-compatible-chat-completions",
        base_url="https://api.groq.com/openai/v1",
        secret_env="GROQ_API_KEY",
        redaction_label="groq_api_key",
    ),
    "hugging-face": ProviderExpectation(
        provider="hugging-face",
        adapter="hugging-face-openai-compatible-chat-completions",
        base_url="https://router.huggingface.co/v1",
        secret_env="HF_TOKEN",
        redaction_label="hf_token",
    ),
    "openai-compatible": ProviderExpectation(
        provider="openai-compatible",
        adapter="openai-compatible-chat-completions",
        base_url=None,
        secret_env="OPENAI_COMPATIBLE_API_KEY",
        redaction_label="openai_compatible_api_key",
    ),
    "openrouter": ProviderExpectation(
        provider="openrouter",
        adapter="openrouter-openai-compatible-chat-completions",
        base_url="https://openrouter.ai/api/v1",
        secret_env="OPENROUTER_API_KEY",
        redaction_label="openrouter_api_key",
    ),
}


def check_replay(
    replay: dict[str, Any],
    *,
    provider: str,
    model: str,
    base_url: str | None = None,
    timeout_seconds: int = 60,
    max_retries: int = 2,
    secret_value: str | None = None,
) -> list[str]:
    errors: list[str] = []
    expectation = PROVIDERS[provider]
    expected_base_url = (base_url or expectation.base_url or "").rstrip("/")
    expected_timeout_nanos = str(timeout_seconds * 1_000_000_000)

    expect_equal(errors, replay, "workflow_name", "support-triage")
    expect_equal(errors, replay, "schema_version", 1)

    runtime = replay.get("runtime")
    if not isinstance(runtime, dict):
        errors.append("runtime must be an object")
        runtime = {}

    expect_equal(errors, runtime, "provider", expectation.provider, prefix="runtime")
    expect_equal(errors, runtime, "adapter", expectation.adapter, prefix="runtime")
    expect_equal(errors, runtime, "model", model, prefix="runtime")

    parameters = runtime.get("parameters")
    if not isinstance(parameters, dict):
        errors.append("runtime.parameters must be an object")
        parameters = {}

    expect_equal(errors, parameters, "base_url", expected_base_url, prefix="runtime.parameters")
    expect_equal(
        errors,
        parameters,
        "timeout_nanos",
        expected_timeout_nanos,
        prefix="runtime.parameters",
    )
    expect_equal(
        errors,
        parameters,
        "max_retries",
        str(max_retries),
        prefix="runtime.parameters",
    )

    steps = replay.get("steps")
    if not isinstance(steps, list):
        errors.append("steps must be an array")
        steps = []
    elif len(steps) != 2:
        errors.append(f"steps length mismatch: expected 2, got {len(steps)}")

    redaction_marker = f"[REDACTED:{expectation.redaction_label}]"
    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            errors.append(f"steps[{index}] must be an object")
            continue
        output = step.get("output")
        if not isinstance(output, str) or not output:
            errors.append(f"steps[{index}].output must be a non-empty string")
        elif redaction_marker in output:
            errors.append(f"steps[{index}].output contains redaction marker {redaction_marker}")

    if secret_value:
        serialized = json.dumps(replay, sort_keys=True)
        if secret_value in serialized:
            errors.append(f"replay contains secret value from {expectation.secret_env}")

    return errors


def expect_equal(
    errors: list[str],
    mapping: dict[str, Any],
    key: str,
    expected: Any,
    *,
    prefix: str | None = None,
) -> None:
    actual = mapping.get(key)
    label = f"{prefix}.{key}" if prefix else key
    if actual != expected:
        errors.append(f"{label} mismatch: expected {expected!r}, got {actual!r}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replay", required=True, type=Path)
    parser.add_argument("--provider", required=True, choices=sorted(PROVIDERS))
    parser.add_argument("--model", required=True)
    parser.add_argument("--base-url")
    parser.add_argument("--timeout-seconds", type=positive_int, default=60)
    parser.add_argument("--max-retries", type=bounded_retries, default=2)
    parser.add_argument("--secret-env")
    return parser.parse_args()


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def bounded_retries(value: str) -> int:
    parsed = int(value)
    if parsed < 0 or parsed > 20:
        raise argparse.ArgumentTypeError("must be between 0 and 20")
    return parsed


def main() -> int:
    args = parse_args()
    replay = json.loads(args.replay.read_text(encoding="utf-8"))
    if not isinstance(replay, dict):
        print("replay must be a JSON object", file=sys.stderr)
        return 1

    secret_value = None
    if args.secret_env:
        import os

        secret_value = os.environ.get(args.secret_env)

    errors = check_replay(
        replay,
        provider=args.provider,
        model=args.model,
        base_url=args.base_url,
        timeout_seconds=args.timeout_seconds,
        max_retries=args.max_retries,
        secret_value=secret_value,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
