#!/usr/bin/env python3
"""Validate the committed provider credential example file."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


EXPECTED_ENV_VARS = {
    "GEMINI_API_KEY",
    "GROQ_API_KEY",
    "HF_TOKEN",
    "OPENROUTER_API_KEY",
    "OPENAI_COMPATIBLE_API_KEY",
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

    errors = check_env_example(args.root.resolve())
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_env_example(root: Path) -> list[str]:
    env_example = root / ".env.example"
    if not env_example.is_file():
        return [".env.example is missing"]

    try:
        assignments = parse_assignments(env_example)
    except ValueError as error:
        return [str(error)]
    errors: list[str] = []
    missing = sorted(EXPECTED_ENV_VARS.difference(assignments))
    unexpected = sorted(set(assignments).difference(EXPECTED_ENV_VARS))
    populated = sorted(name for name, value in assignments.items() if value)

    if missing:
        errors.append(".env.example is missing provider variable(s): " + ", ".join(missing))
    if unexpected:
        errors.append(".env.example contains unexpected variable(s): " + ", ".join(unexpected))
    if populated:
        errors.append(".env.example must keep committed values blank: " + ", ".join(populated))

    return errors


def parse_assignments(path: Path) -> dict[str, str]:
    assignments: dict[str, str] = {}
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError(f"{path}:{line_number}: expected KEY=VALUE assignment")
        name, value = line.split("=", maxsplit=1)
        assignments[name.strip()] = value.strip()
    return assignments


if __name__ == "__main__":
    sys.exit(main())
