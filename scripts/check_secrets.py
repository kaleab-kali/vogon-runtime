#!/usr/bin/env python3
"""Scan tracked text files for committed secret-looking values."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


MAX_TEXT_BYTES = 1_000_000


@dataclass(frozen=True)
class SecretPattern:
    name: str
    pattern: re.Pattern[str]


SECRET_PATTERNS = [
    SecretPattern("AWS access key id", re.compile(r"\b(?:A3T|AKIA|ASIA)[A-Z0-9]{16}\b")),
    SecretPattern("GitHub token", re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{36,}\b")),
    SecretPattern("Google API key", re.compile(r"\bAIza[0-9A-Za-z_-]{35}\b")),
    SecretPattern("Hugging Face token", re.compile(r"\bhf_[A-Za-z0-9]{30,}\b")),
    SecretPattern("OpenAI API key", re.compile(r"\bsk-[A-Za-z0-9]{20,}\b")),
    SecretPattern("OpenRouter API key", re.compile(r"\bsk-or-v1-[A-Za-z0-9]{32,}\b")),
    SecretPattern(
        "Slack token",
        re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{20,}\b"),
    ),
]
PROVIDER_CREDENTIAL_VARS = {
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENAI_COMPATIBLE_API_KEY",
    "OPENROUTER_API_KEY",
}
PROVIDER_ASSIGNMENT_RE = re.compile(
    r"(?<![A-Z0-9_{])("
    + "|".join(sorted(PROVIDER_CREDENTIAL_VARS))
    + r")\s*[:=]\s*([^\s#]+)?"
)
PLACEHOLDER_VALUES = {
    "",
    "...",
    "''",
    '""',
    "<token>",
    "<api-key>",
    "<api_key>",
    "<secret>",
    "changeme",
    "change-me",
    "your-token",
    "your-api-key",
    "your_api_key",
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root. Defaults to the current directory.",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    findings = check_repository(root)
    for finding in findings:
        print(finding, file=sys.stderr)
    return 1 if findings else 0


def check_repository(root: Path) -> list[str]:
    findings: list[str] = []
    for path in tracked_files(root):
        text = read_text_file(path)
        if text is None:
            continue
        for line_number, line in enumerate(text.splitlines(), start=1):
            for secret_pattern in SECRET_PATTERNS:
                if secret_pattern.pattern.search(line):
                    findings.append(format_finding(root, path, line_number, secret_pattern.name))
            provider_assignment = find_provider_secret_assignment(line)
            if provider_assignment:
                findings.append(format_finding(root, path, line_number, provider_assignment))
    return findings


def find_provider_secret_assignment(line: str) -> str | None:
    match = PROVIDER_ASSIGNMENT_RE.search(line)
    if not match:
        return None

    name = match.group(1)
    value = normalize_assignment_value(match.group(2) or "")
    if is_allowed_placeholder_value(value):
        return None
    return f"committed {name} value"


def normalize_assignment_value(value: str) -> str:
    return value.strip().strip(",").strip("\"'")


def is_allowed_placeholder_value(value: str) -> bool:
    lowered = value.lower()
    return (
        lowered in PLACEHOLDER_VALUES
        or value.startswith("${{")
        or value.startswith("$")
        or "..." in value
        or lowered.startswith("your-")
        or lowered.startswith("<")
    )


def tracked_files(root: Path) -> list[Path]:
    git_dir = root / ".git"
    if git_dir.exists():
        output = subprocess.check_output(
            ["git", "ls-files"],
            cwd=root,
            text=True,
            encoding="utf-8",
        )
        return [root / line for line in output.splitlines() if line]

    ignored_parts = {".git", "target"}
    return sorted(
        path
        for path in root.rglob("*")
        if path.is_file() and not ignored_parts.intersection(path.relative_to(root).parts)
    )


def read_text_file(path: Path) -> str | None:
    try:
        data = path.read_bytes()
    except OSError:
        return None
    if len(data) > MAX_TEXT_BYTES or b"\0" in data:
        return None
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def format_finding(root: Path, path: Path, line_number: int, pattern_name: str) -> str:
    relative_path = path.relative_to(root).as_posix()
    return f"{relative_path}:{line_number}: possible {pattern_name}"


if __name__ == "__main__":
    sys.exit(main())
