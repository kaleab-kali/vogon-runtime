#!/usr/bin/env python3
"""Validate published schema files used by contributors and editors."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


EXPECTED_SCHEMAS = {
    "schemas/workflow.schema.json": {
        "title": "Vogon Workflow",
        "required": ["name", "steps"],
    },
    "schemas/replay.schema.json": {
        "title": "Vogon Replay",
        "required": ["schema_version", "workflow_name", "runtime", "run_hash", "steps"],
    },
}
SCHEMA_DRAFT = "https://json-schema.org/draft/2020-12/schema"


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
    errors: list[str] = []
    for relative_path, expected in EXPECTED_SCHEMAS.items():
        path = root / relative_path
        if not path.exists():
            errors.append(f"{relative_path}: missing schema file")
            continue

        try:
            schema = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            errors.append(f"{relative_path}: invalid JSON: {error.msg}")
            continue

        if not isinstance(schema, dict):
            errors.append(f"{relative_path}: schema root must be an object")
            continue

        errors.extend(check_schema(relative_path, schema, expected))

    return errors


def check_schema(
    relative_path: str,
    schema: dict[str, Any],
    expected: dict[str, Any],
) -> list[str]:
    errors: list[str] = []

    if schema.get("$schema") != SCHEMA_DRAFT:
        errors.append(f"{relative_path}: schema draft must be {SCHEMA_DRAFT!r}")
    if schema.get("title") != expected["title"]:
        errors.append(f"{relative_path}: title must be {expected['title']!r}")
    if schema.get("type") != "object":
        errors.append(f"{relative_path}: root type must be 'object'")
    if schema.get("additionalProperties") is not False:
        errors.append(f"{relative_path}: root additionalProperties must be false")
    if schema.get("required") != expected["required"]:
        errors.append(f"{relative_path}: required fields must match documented format")

    properties = schema.get("properties")
    if not isinstance(properties, dict):
        errors.append(f"{relative_path}: missing root properties")
        return errors

    for field in expected["required"]:
        if field not in properties:
            errors.append(f"{relative_path}: missing `{field}` property")

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
