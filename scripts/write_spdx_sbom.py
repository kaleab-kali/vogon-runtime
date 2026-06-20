#!/usr/bin/env python3
"""Write a minimal SPDX 2.3 JSON SBOM from Cargo metadata."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


NOASSERTION = "NOASSERTION"
SPDX_VERSION = "SPDX-2.3"
DATA_LICENSE = "CC0-1.0"


def main() -> int:
    return main_with_args(sys.argv[1:])


def main_with_args(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--document-name", required=True)
    parser.add_argument("--namespace", required=True)
    parser.add_argument(
        "--created",
        default=None,
        help="Creation timestamp. Defaults to SOURCE_DATE_EPOCH or current UTC time.",
    )
    args = parser.parse_args(argv)

    metadata = json.loads(args.metadata.read_text(encoding="utf-8-sig"))
    document = build_document(
        metadata,
        document_name=args.document_name,
        namespace=args.namespace,
        created=args.created or created_timestamp(),
    )
    args.output.write_text(
        json.dumps(document, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


def created_timestamp() -> str:
    source_date_epoch = None
    try:
        import os

        source_date_epoch = os.environ.get("SOURCE_DATE_EPOCH")
        if source_date_epoch is not None:
            timestamp = dt.datetime.fromtimestamp(
                int(source_date_epoch),
                tz=dt.UTC,
            )
            return timestamp.strftime("%Y-%m-%dT%H:%M:%SZ")
    except (OverflowError, ValueError) as error:
        raise SystemExit(f"invalid SOURCE_DATE_EPOCH `{source_date_epoch}`: {error}") from error

    return dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def build_document(
    metadata: dict[str, Any],
    *,
    document_name: str,
    namespace: str,
    created: str,
) -> dict[str, Any]:
    packages_by_id = {package["id"]: package for package in metadata["packages"]}
    root_ids = sorted(root_package_ids(metadata))
    root_spdx_ids = [package_spdx_id(packages_by_id[package_id]) for package_id in root_ids]

    packages = [document_package()]
    packages.extend(
        package_document(package) for package in sorted(metadata["packages"], key=package_sort_key)
    )

    relationships = []
    for root_spdx_id in root_spdx_ids:
        relationships.append(
            relationship("SPDXRef-DOCUMENT", "DESCRIBES", root_spdx_id),
        )
        relationships.append(
            relationship("SPDXRef-Package-vogon-runtime-source", "GENERATES", root_spdx_id),
        )

    for node in sorted(metadata["resolve"]["nodes"], key=lambda item: item["id"]):
        source = package_spdx_id(packages_by_id[node["id"]])
        for dependency_id in sorted(dependency["pkg"] for dependency in node["deps"]):
            relationships.append(
                relationship(source, "DEPENDS_ON", package_spdx_id(packages_by_id[dependency_id])),
            )

    return {
        "spdxVersion": SPDX_VERSION,
        "dataLicense": DATA_LICENSE,
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": document_name,
        "documentNamespace": namespace,
        "creationInfo": {
            "created": created,
            "creators": ["Tool: vogon-runtime scripts/write_spdx_sbom.py"],
        },
        "packages": packages,
        "relationships": relationships,
    }


def document_package() -> dict[str, Any]:
    return {
        "SPDXID": "SPDXRef-Package-vogon-runtime-source",
        "name": "vogon-runtime-source",
        "downloadLocation": "git+https://github.com/kaleab-kali/vogon-runtime.git",
        "filesAnalyzed": False,
        "licenseConcluded": NOASSERTION,
        "licenseDeclared": NOASSERTION,
        "copyrightText": NOASSERTION,
    }


def package_document(package: dict[str, Any]) -> dict[str, Any]:
    return {
        "SPDXID": package_spdx_id(package),
        "name": package["name"],
        "versionInfo": package["version"],
        "downloadLocation": download_location(package),
        "filesAnalyzed": False,
        "licenseConcluded": NOASSERTION,
        "licenseDeclared": package.get("license") or NOASSERTION,
        "copyrightText": NOASSERTION,
    }


def download_location(package: dict[str, Any]) -> str:
    source = package.get("source")
    if source:
        if source.startswith("registry+"):
            return source.removeprefix("registry+")
        return source

    manifest_path = package.get("manifest_path")
    if manifest_path:
        return f"file://{manifest_path}"

    return NOASSERTION


def root_package_ids(metadata: dict[str, Any]) -> set[str]:
    root_id = metadata["resolve"].get("root")
    if root_id:
        return {root_id}

    workspace_members = metadata.get("workspace_members") or []
    return set(workspace_members)


def relationship(
    source: str,
    relationship_type: str,
    target: str,
) -> dict[str, str]:
    return {
        "spdxElementId": source,
        "relationshipType": relationship_type,
        "relatedSpdxElement": target,
    }


def package_spdx_id(package: dict[str, Any]) -> str:
    name = sanitize_spdx_ref(package["name"])
    version = sanitize_spdx_ref(package["version"])
    package_id = hashlib.sha256(package["id"].encode("utf-8")).hexdigest()[:12]
    return f"SPDXRef-Package-{name}-{version}-{package_id}"


def sanitize_spdx_ref(value: str) -> str:
    sanitized = re.sub(r"[^A-Za-z0-9.-]", "-", value)
    sanitized = re.sub(r"-+", "-", sanitized).strip("-")
    return sanitized or "unknown"


def package_sort_key(package: dict[str, Any]) -> tuple[str, str, str]:
    return (package["name"], package["version"], package["id"])


if __name__ == "__main__":
    sys.exit(main())
