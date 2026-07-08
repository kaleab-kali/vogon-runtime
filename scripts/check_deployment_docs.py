#!/usr/bin/env python3
"""Validate provider-backed deployment documentation examples."""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path


PROVIDER_CREDENTIALS_MARKER = "## Provider Credentials"


@dataclass(frozen=True)
class ProviderDeploymentExample:
    provider: str
    env_var: str


EXPECTED_PROVIDER_EXAMPLES = [
    ProviderDeploymentExample(provider="gemini", env_var="GEMINI_API_KEY"),
    ProviderDeploymentExample(provider="openai-compatible", env_var="OPENAI_COMPATIBLE_API_KEY"),
    ProviderDeploymentExample(provider="groq", env_var="GROQ_API_KEY"),
    ProviderDeploymentExample(provider="hugging-face", env_var="HF_TOKEN"),
    ProviderDeploymentExample(provider="openrouter", env_var="OPENROUTER_API_KEY"),
]


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
    path = root / "docs" / "deployment.md"
    if not path.exists():
        return ["docs/deployment.md: missing deployment documentation"]

    text = path.read_text(encoding="utf-8")
    provider_section = extract_section(text, PROVIDER_CREDENTIALS_MARKER)
    if not provider_section:
        return ["docs/deployment.md: missing Provider Credentials section"]

    errors: list[str] = []
    for example in EXPECTED_PROVIDER_EXAMPLES:
        if f"-e {example.env_var}" not in provider_section:
            errors.append(
                "docs/deployment.md: "
                f"missing container env example for {example.env_var}"
            )
        if f"--provider {example.provider}" not in provider_section:
            errors.append(
                "docs/deployment.md: "
                f"missing container run example for provider `{example.provider}`"
            )

    return errors


def extract_section(text: str, marker: str) -> str:
    lines = text.splitlines()
    try:
        start = lines.index(marker)
    except ValueError:
        return ""

    section_lines: list[str] = []
    for line in lines[start + 1 :]:
        if line.startswith("## "):
            break
        section_lines.append(line)
    return "\n".join(section_lines)


if __name__ == "__main__":
    raise SystemExit(main())
