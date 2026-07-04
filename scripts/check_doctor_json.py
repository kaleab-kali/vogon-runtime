#!/usr/bin/env python3
"""Validate machine-readable Vogon doctor diagnostics."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any


EXPECTED_PROVIDER_USAGE_URLS = {
    "gemini": "https://ai.google.dev/gemini-api/docs/pricing",
    "groq": "https://console.groq.com/docs/rate-limits",
    "hugging-face": "https://huggingface.co/docs/inference-providers/pricing",
    "openrouter": "https://openrouter.ai/pricing",
}

EXPECTED_PROVIDER_NULL_USAGE_URLS = {
    "deterministic",
    "openai-compatible",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()

    errors = check_output(sys.stdin.read())
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_output(output: str) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"doctor JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["doctor JSON root must be an object"]

    errors: list[str] = []

    if data.get("status") != "ok":
        errors.append("doctor status must be ok")

    checks = data.get("checks")
    if not isinstance(checks, list):
        errors.append("doctor checks must be an array")
    elif not any(
        isinstance(check, dict)
        and check.get("name") == "deterministic_runtime"
        and check.get("status") == "ok"
        for check in checks
    ):
        errors.append("doctor checks must include ok deterministic_runtime")

    providers = data.get("providers")
    if not isinstance(providers, list):
        errors.append("doctor providers must be an array")
        return errors

    providers_by_name = {
        provider.get("name"): provider
        for provider in providers
        if isinstance(provider, dict) and isinstance(provider.get("name"), str)
    }

    for name, expected_url in EXPECTED_PROVIDER_USAGE_URLS.items():
        provider = providers_by_name.get(name)
        if provider is None:
            errors.append(f"doctor providers must include {name}")
            continue
        if provider.get("usage_url") != expected_url:
            errors.append(
                f"doctor provider {name} usage_url mismatch: "
                f"expected {expected_url}, got {format_json_value(provider.get('usage_url'))}"
            )

    for name in sorted(EXPECTED_PROVIDER_NULL_USAGE_URLS):
        provider = providers_by_name.get(name)
        if provider is None:
            errors.append(f"doctor providers must include {name}")
            continue
        if provider.get("usage_url") is not None:
            errors.append(
                f"doctor provider {name} usage_url must be null, "
                f"got {format_json_value(provider.get('usage_url'))}"
            )

    return errors


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
