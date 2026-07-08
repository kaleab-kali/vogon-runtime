#!/usr/bin/env python3
"""Validate machine-readable Vogon provider diagnostics."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any


EXPECTED_PROVIDERS = {
    "deterministic": {
        "default": True,
        "credential_env": None,
        "credential_configured": None,
        "default_base_url": None,
        "default_model": None,
        "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#deterministic",
        "usage_url": None,
    },
    "gemini": {
        "default": False,
        "credential_env": "GEMINI_API_KEY",
        "credential_configured": "bool_or_null",
        "default_base_url": None,
        "default_model": "gemini-3.1-flash-lite",
        "documentation_url": "https://ai.google.dev/gemini-api/docs",
        "usage_url": "https://ai.google.dev/gemini-api/docs/pricing",
    },
    "groq": {
        "default": False,
        "credential_env": "GROQ_API_KEY",
        "credential_configured": "bool_or_null",
        "default_base_url": "https://api.groq.com/openai/v1",
        "default_model": "llama-3.1-8b-instant",
        "documentation_url": "https://console.groq.com/docs/openai",
        "usage_url": "https://console.groq.com/docs/rate-limits",
    },
    "hugging-face": {
        "default": False,
        "credential_env": "HF_TOKEN",
        "credential_configured": "bool_or_null",
        "default_base_url": "https://router.huggingface.co/v1",
        "default_model": "openai/gpt-oss-120b:fastest",
        "documentation_url": "https://huggingface.co/docs/inference-providers",
        "usage_url": "https://huggingface.co/docs/inference-providers/pricing",
    },
    "openrouter": {
        "default": False,
        "credential_env": "OPENROUTER_API_KEY",
        "credential_configured": "bool_or_null",
        "default_base_url": "https://openrouter.ai/api/v1",
        "default_model": "openrouter/free",
        "documentation_url": "https://openrouter.ai/docs",
        "usage_url": "https://openrouter.ai/pricing",
    },
    "openai-compatible": {
        "default": False,
        "credential_env": "OPENAI_COMPATIBLE_API_KEY",
        "credential_configured": "bool_or_null",
        "default_base_url": "https://router.huggingface.co/v1",
        "default_model": "openai/gpt-oss-120b:fastest",
        "documentation_url": "https://github.com/kaleab-kali/vogon-runtime/blob/main/docs/providers.md#openai-compatible",
        "usage_url": None,
    },
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
        return [f"providers JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["providers JSON root must be an object"]

    providers = data.get("providers")
    if not isinstance(providers, list):
        return ["providers must be an array"]

    errors: list[str] = []
    providers_by_name: dict[str, dict[str, Any]] = {}
    for index, provider in enumerate(providers):
        if not isinstance(provider, dict):
            errors.append(f"provider at index {index} must be an object")
            continue
        name = provider.get("name")
        if not isinstance(name, str):
            errors.append(f"provider at index {index} must have string name")
            continue
        if name in providers_by_name:
            errors.append(f"duplicate provider {name}")
            continue
        providers_by_name[name] = provider

    expected_names = set(EXPECTED_PROVIDERS)
    actual_names = set(providers_by_name)
    for name in sorted(expected_names - actual_names):
        errors.append(f"providers must include {name}")
    for name in sorted(actual_names - expected_names):
        errors.append(f"providers must not include unexpected provider {name}")

    default_count = 0
    for name, expected in EXPECTED_PROVIDERS.items():
        provider = providers_by_name.get(name)
        if provider is None:
            continue
        if provider.get("enabled") not in {True, False}:
            errors.append(f"provider {name} enabled must be boolean")
        if provider.get("default") is True:
            default_count += 1
        for field, expected_value in expected.items():
            validate_field(errors, name, provider, field, expected_value)

    if default_count != 1:
        errors.append(f"exactly one provider must be default, found {default_count}")

    return errors


def validate_field(
    errors: list[str],
    name: str,
    provider: dict[str, Any],
    field: str,
    expected_value: object,
) -> None:
    actual_value = provider.get(field)
    if expected_value == "bool_or_null":
        if actual_value is not None and not isinstance(actual_value, bool):
            errors.append(
                f"provider {name} {field} must be boolean or null, "
                f"got {format_json_value(actual_value)}"
            )
        return

    if actual_value != expected_value:
        errors.append(
            f"provider {name} {field} mismatch: expected "
            f"{format_json_value(expected_value)}, got {format_json_value(actual_value)}"
        )


def format_json_value(value: object) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
