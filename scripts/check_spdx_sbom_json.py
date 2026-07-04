#!/usr/bin/env python3
"""Validate SPDX SBOM JSON used as a release artifact."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sbom_file", type=Path, help="SPDX SBOM JSON file.")
    parser.add_argument("--expected-name", help="Expected SPDX document name.")
    parser.add_argument(
        "--expected-package",
        action="append",
        default=[],
        help="Package name expected in the SBOM. May be repeated.",
    )
    args = parser.parse_args()

    errors = check_file(
        args.sbom_file,
        expected_name=args.expected_name,
        expected_packages=args.expected_package,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_file(
    path: Path,
    *,
    expected_name: str | None = None,
    expected_packages: list[str] | None = None,
) -> list[str]:
    try:
        output = path.read_text(encoding="utf-8-sig")
    except OSError as error:
        return [f"SPDX SBOM JSON file cannot be read: {error}"]

    return check_output(
        output,
        expected_name=expected_name,
        expected_packages=expected_packages,
    )


def check_output(
    output: str,
    *,
    expected_name: str | None = None,
    expected_packages: list[str] | None = None,
) -> list[str]:
    try:
        data = json.loads(output)
    except json.JSONDecodeError as error:
        return [f"SPDX SBOM JSON is invalid: {error}"]

    if not isinstance(data, dict):
        return ["SPDX SBOM JSON root must be an object"]

    errors: list[str] = []
    if data.get("spdxVersion") != "SPDX-2.3":
        errors.append(
            "SPDX SBOM spdxVersion mismatch: "
            f"expected SPDX-2.3, got {format_json_value(data.get('spdxVersion'))}"
        )
    if data.get("dataLicense") != "CC0-1.0":
        errors.append(
            "SPDX SBOM dataLicense mismatch: "
            f"expected CC0-1.0, got {format_json_value(data.get('dataLicense'))}"
        )
    if data.get("SPDXID") != "SPDXRef-DOCUMENT":
        errors.append("SPDX SBOM SPDXID must be SPDXRef-DOCUMENT")

    name = data.get("name")
    if not isinstance(name, str) or not name:
        errors.append("SPDX SBOM name must be a non-empty string")
    elif expected_name is not None and name != expected_name:
        errors.append(
            "SPDX SBOM name mismatch: "
            f"expected {expected_name}, got {format_json_value(name)}"
        )

    namespace = data.get("documentNamespace")
    if not isinstance(namespace, str) or not namespace.startswith("https://"):
        errors.append("SPDX SBOM documentNamespace must be an HTTPS URL")

    creation_info = data.get("creationInfo")
    if not isinstance(creation_info, dict):
        errors.append("SPDX SBOM creationInfo must be an object")
    else:
        creators = creation_info.get("creators")
        if not isinstance(creators, list) or not any(
            isinstance(creator, str)
            and creator == "Tool: vogon-runtime scripts/write_spdx_sbom.py"
            for creator in creators
        ):
            errors.append("SPDX SBOM creators must include the Vogon SBOM writer")

    packages = data.get("packages")
    if not isinstance(packages, list) or not packages:
        errors.append("SPDX SBOM packages must be a non-empty array")
        packages = []

    package_names: set[str] = set()
    for index, package in enumerate(packages, start=1):
        if not isinstance(package, dict):
            errors.append(f"SPDX SBOM package {index} must be an object")
            continue
        package_name = require_string(package, "name", f"SPDX SBOM package {index}", errors)
        require_string(package, "SPDXID", f"SPDX SBOM package {index}", errors)
        require_string(package, "downloadLocation", f"SPDX SBOM package {index}", errors)
        if package_name is not None:
            package_names.add(package_name)

    expected_packages = expected_packages or []
    for package_name in expected_packages:
        if package_name not in package_names:
            errors.append(
                "SPDX SBOM package missing: "
                f"expected {package_name}, got {format_json_value(sorted(package_names))}"
            )

    relationships = data.get("relationships")
    if not isinstance(relationships, list) or not relationships:
        errors.append("SPDX SBOM relationships must be a non-empty array")
    else:
        if not any(
            isinstance(relationship, dict)
            and relationship.get("relationshipType") == "DESCRIBES"
            for relationship in relationships
        ):
            errors.append("SPDX SBOM relationships must include DESCRIBES")
        if not any(
            isinstance(relationship, dict)
            and relationship.get("relationshipType") == "DEPENDS_ON"
            for relationship in relationships
        ):
            errors.append("SPDX SBOM relationships must include DEPENDS_ON")

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
