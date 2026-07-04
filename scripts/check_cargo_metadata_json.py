#!/usr/bin/env python3
"""Validate Cargo metadata JSON used as a release artifact."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata_file", type=Path, help="Cargo metadata JSON file.")
    parser.add_argument(
        "--expected-workspace-package",
        action="append",
        default=[],
        help="Workspace package name expected in the metadata. May be repeated.",
    )
    args = parser.parse_args()

    errors = check_file(
        args.metadata_file,
        expected_workspace_packages=args.expected_workspace_package,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_file(
    path: Path,
    *,
    expected_workspace_packages: list[str] | None = None,
) -> list[str]:
    try:
        output = path.read_text(encoding="utf-8-sig")
    except OSError as error:
        return [f"Cargo metadata JSON file cannot be read: {error}"]

    return check_output(
        output,
        expected_workspace_packages=expected_workspace_packages,
    )


def check_output(
    output: str,
    *,
    expected_workspace_packages: list[str] | None = None,
) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"Cargo metadata JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["Cargo metadata JSON root must be an object"]

    errors: list[str] = []
    packages = data.get("packages")
    if not isinstance(packages, list) or not packages:
        errors.append("Cargo metadata JSON packages must be a non-empty array")
        packages = []

    package_ids: set[str] = set()
    package_names_by_id: dict[str, str] = {}
    for index, package in enumerate(packages, start=1):
        if not isinstance(package, dict):
            errors.append(f"Cargo metadata package {index} must be an object")
            continue
        package_id = require_string(package, "id", f"Cargo metadata package {index}", errors)
        package_name = require_string(
            package,
            "name",
            f"Cargo metadata package {index}",
            errors,
        )
        require_string(package, "version", f"Cargo metadata package {index}", errors)
        require_string(package, "manifest_path", f"Cargo metadata package {index}", errors)
        if package_id is not None:
            package_ids.add(package_id)
            if package_name is not None:
                package_names_by_id[package_id] = package_name

    workspace_members = data.get("workspace_members")
    if not isinstance(workspace_members, list) or not workspace_members:
        errors.append("Cargo metadata JSON workspace_members must be a non-empty array")
        workspace_members = []

    for index, member_id in enumerate(workspace_members, start=1):
        if not isinstance(member_id, str) or not member_id:
            errors.append(
                f"Cargo metadata workspace member {index} must be a non-empty string"
            )
        elif package_ids and member_id not in package_ids:
            errors.append(f"Cargo metadata workspace member {index} is missing from packages")

    expected_workspace_packages = expected_workspace_packages or []
    workspace_package_names = {
        package_names_by_id[member_id]
        for member_id in workspace_members
        if isinstance(member_id, str) and member_id in package_names_by_id
    }
    for package_name in expected_workspace_packages:
        if package_name not in workspace_package_names:
            errors.append(
                "Cargo metadata workspace package missing: "
                f"expected {package_name}, got {format_json_value(sorted(workspace_package_names))}"
            )

    resolve = data.get("resolve")
    if not isinstance(resolve, dict):
        errors.append("Cargo metadata JSON resolve must be an object")
        resolve = {}

    nodes = resolve.get("nodes")
    if not isinstance(nodes, list) or not nodes:
        errors.append("Cargo metadata JSON resolve.nodes must be a non-empty array")
        nodes = []

    for index, node in enumerate(nodes, start=1):
        if not isinstance(node, dict):
            errors.append(f"Cargo metadata resolve node {index} must be an object")
            continue
        node_id = require_string(node, "id", f"Cargo metadata resolve node {index}", errors)
        deps = node.get("deps")
        if not isinstance(deps, list):
            errors.append(f"Cargo metadata resolve node {index} deps must be an array")
        if node_id is not None and package_ids and node_id not in package_ids:
            errors.append(f"Cargo metadata resolve node {index} is missing from packages")

    return errors


def require_string(
    data: dict[str, Any],
    field: str,
    context: str,
    errors: list[str],
) -> str | None:
    value = data.get(field)
    if not isinstance(value, str) or not value:
        errors.append(f"{context} {field} must be a non-empty string")
        return None
    return value


def format_json_value(value: Any) -> str:
    return json.dumps(value, sort_keys=True)


if __name__ == "__main__":
    sys.exit(main())
