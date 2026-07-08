#!/usr/bin/env python3
"""Validate published schema files used by contributors and editors."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
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
IDENTIFIER_PATTERN_DESCRIPTION = "ASCII letters, digits, underscores, and hyphens"


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

    errors.extend(check_workflow_fixtures(root))
    errors.extend(check_replay_fixtures(root))

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


def check_workflow_fixtures(root: Path) -> list[str]:
    workflows_dir = root / "fixtures" / "workflows"
    if not workflows_dir.exists():
        return ["fixtures/workflows: missing workflow fixtures"]

    errors: list[str] = []
    fixture_paths = sorted(workflows_dir.glob("*.toml"))
    if not fixture_paths:
        errors.append("fixtures/workflows: missing workflow fixture files")

    for path in fixture_paths:
        relative_path = path.relative_to(root).as_posix()
        try:
            workflow = tomllib.loads(path.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as error:
            errors.append(f"{relative_path}: invalid TOML: {error}")
            continue

        if not isinstance(workflow, dict):
            errors.append(f"{relative_path}: workflow fixture must be an object")
            continue

        errors.extend(check_workflow_document(relative_path, workflow))

    return errors


def check_workflow_document(relative_path: str, workflow: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    allowed_fields = {"name", "steps"}
    for field in sorted(set(workflow) - allowed_fields):
        errors.append(f"{relative_path}: unknown workflow field `{field}`")

    name = workflow.get("name")
    if not is_identifier(name):
        errors.append(
            f"{relative_path}: workflow name must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        )

    steps = workflow.get("steps")
    if not isinstance(steps, list) or not steps:
        errors.append(f"{relative_path}: workflow steps must be a non-empty list")
        return errors

    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            errors.append(f"{relative_path}: workflow step {index} must be an object")
            continue
        for field in sorted(set(step) - {"id", "prompt"}):
            errors.append(
                f"{relative_path}: workflow step {index} has unknown field `{field}`"
            )
        if not is_identifier(step.get("id")):
            errors.append(
                f"{relative_path}: workflow step {index} id must use {IDENTIFIER_PATTERN_DESCRIPTION}"
            )
        prompt = step.get("prompt")
        if not isinstance(prompt, str) or prompt == "":
            errors.append(
                f"{relative_path}: workflow step {index} prompt must be non-empty"
            )

    return errors


def check_replay_fixtures(root: Path) -> list[str]:
    replays_dir = root / "fixtures" / "replays"
    if not replays_dir.exists():
        return ["fixtures/replays: missing replay fixtures"]

    errors: list[str] = []
    fixture_paths = sorted(replays_dir.glob("*.replay.json"))
    if not fixture_paths:
        errors.append("fixtures/replays: missing replay fixture files")

    for path in fixture_paths:
        relative_path = path.relative_to(root).as_posix()
        try:
            replay = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            errors.append(f"{relative_path}: invalid JSON: {error.msg}")
            continue

        if not isinstance(replay, dict):
            errors.append(f"{relative_path}: replay fixture must be an object")
            continue

        errors.extend(check_replay_document(relative_path, replay))

    return errors


def check_replay_document(relative_path: str, replay: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    allowed_fields = {"schema_version", "workflow_name", "runtime", "run_hash", "steps"}
    required_fields = [
        "schema_version",
        "workflow_name",
        "runtime",
        "run_hash",
        "steps",
    ]
    for field in sorted(set(replay) - allowed_fields):
        errors.append(f"{relative_path}: unknown replay field `{field}`")
    for field in required_fields:
        if field not in replay:
            errors.append(f"{relative_path}: missing replay field `{field}`")

    if replay.get("schema_version") != 1:
        errors.append(f"{relative_path}: replay schema_version must be 1")
    if not is_identifier(replay.get("workflow_name")):
        errors.append(
            f"{relative_path}: replay workflow_name must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        )
    if not is_sha256(replay.get("run_hash")):
        errors.append(f"{relative_path}: replay run_hash must be lowercase sha256")

    runtime = replay.get("runtime")
    if not isinstance(runtime, dict):
        errors.append(f"{relative_path}: replay runtime must be an object")
    else:
        errors.extend(check_runtime_metadata(relative_path, runtime))

    steps = replay.get("steps")
    if not isinstance(steps, list) or not steps:
        errors.append(f"{relative_path}: replay steps must be a non-empty list")
        return errors

    for index, step in enumerate(steps):
        if not isinstance(step, dict):
            errors.append(f"{relative_path}: replay step {index} must be an object")
            continue
        errors.extend(check_replay_step(relative_path, index, step))

    return errors


def check_runtime_metadata(relative_path: str, runtime: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    allowed_fields = {
        "provider",
        "adapter",
        "adapter_version",
        "model",
        "cache_identity",
        "parameters",
    }
    required_fields = ["provider", "adapter", "adapter_version", "cache_identity"]
    for field in sorted(set(runtime) - allowed_fields):
        errors.append(f"{relative_path}: unknown runtime field `{field}`")
    for field in required_fields:
        if not is_non_empty_string(runtime.get(field)):
            errors.append(f"{relative_path}: runtime `{field}` must be non-empty")

    model = runtime.get("model")
    if model is not None and not is_non_empty_string(model):
        errors.append(
            f"{relative_path}: runtime `model` must be non-empty when present"
        )

    parameters = runtime.get("parameters", {})
    if not isinstance(parameters, dict):
        errors.append(f"{relative_path}: runtime `parameters` must be an object")
    else:
        for key, value in parameters.items():
            if not is_non_empty_string(key) or not is_non_empty_string(value):
                errors.append(
                    f"{relative_path}: runtime parameters must use non-empty string keys and values"
                )
                break

    return errors


def check_replay_step(relative_path: str, index: int, step: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    allowed_fields = {"step_id", "input_hash", "output_hash", "output"}
    for field in sorted(set(step) - allowed_fields):
        errors.append(
            f"{relative_path}: replay step {index} has unknown field `{field}`"
        )
    if not is_identifier(step.get("step_id")):
        errors.append(
            f"{relative_path}: replay step {index} id must use {IDENTIFIER_PATTERN_DESCRIPTION}"
        )
    if not is_sha256(step.get("input_hash")):
        errors.append(
            f"{relative_path}: replay step {index} input_hash must be lowercase sha256"
        )
    if not is_sha256(step.get("output_hash")):
        errors.append(
            f"{relative_path}: replay step {index} output_hash must be lowercase sha256"
        )
    if not isinstance(step.get("output"), str):
        errors.append(f"{relative_path}: replay step {index} output must be a string")
    return errors


def is_identifier(value: Any) -> bool:
    return (
        isinstance(value, str)
        and bool(value)
        and all(
            character.isascii() and (character.isalnum() or character in "_-")
            for character in value
        )
    )


def is_non_empty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value)


def is_sha256(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


if __name__ == "__main__":
    raise SystemExit(main())
