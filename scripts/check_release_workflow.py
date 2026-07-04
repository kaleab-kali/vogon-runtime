#!/usr/bin/env python3
"""Validate release workflow artifact and provenance coverage."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


REQUIRED_SNIPPETS = {
    "release workflow name": "name: Release",
    "semantic version tag trigger": '      - "v*.*.*"',
    "manual dispatch trigger": "  workflow_dispatch:",
    "read-only top-level contents permission": "permissions:\n  contents: read",
    "linux artifact job": "  linux-cli:",
    "windows artifact job": "  windows-cli:",
    "container artifact job": "  container-image:",
    "release artifact download smoke job": "  release-artifact-smoke:",
    "publish release job": "  publish-release:",
    "linux release build": "cargo build --release -p vogon-cli --locked",
    "linux archive": "vogon-${{ github.ref_name }}-linux-x86_64.tar.gz",
    "windows archive": "vogon-${{ github.ref_name }}-windows-x86_64.zip",
    "container archive": "vogon-${{ github.ref_name }}-container-image.tar.gz",
    "dependency metadata": "cargo metadata --locked --format-version 1",
    "SPDX SBOM writer": "python3 scripts/write_spdx_sbom.py",
    "doctor JSON validator": "scripts/check_doctor_json.py",
    "cache JSON validator": "scripts/check_cache_json.py",
    "workflow check JSON validator": "scripts/check_workflow_json.py",
    "verify JSON validator": "scripts/check_verify_json.py",
    "trace JSONL validator": "scripts/check_trace_jsonl.py",
    "SPDX version validation": "assert data['spdxVersion'] == 'SPDX-2.3'",
    "Linux checksum": "vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256",
    "Windows checksum": "vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256",
    "container checksum": "vogon-${{ github.ref_name }}-container-image.tar.gz.sha256",
    "metadata checksum": "vogon-${{ github.ref_name }}-cargo-metadata.json.sha256",
    "SBOM checksum": "vogon-${{ github.ref_name }}-cargo-spdx.json.sha256",
    "artifact attestation": "uses: actions/attest@v4",
    "artifact upload": "uses: actions/upload-artifact@v7",
    "artifact download": "uses: actions/download-artifact@v8",
    "missing artifact failure": "if-no-files-found: error",
    "GitHub release creation": "gh release create",
    "container OCI source label smoke": (
        'index .Config.Labels "org.opencontainers.image.source"'
    ),
    "container OCI license label smoke": (
        'index .Config.Labels "org.opencontainers.image.licenses"'
    ),
    "non-root container smoke": 'docker run --rm --entrypoint id "$image" -u',
    "read-only container smoke": "docker run --rm --read-only",
}

REQUIRED_COUNTS = {
    "uses: actions/checkout@v7": 4,
    "uses: actions/attest@v4": 3,
    "uses: actions/upload-artifact@v7": 3,
    "uses: actions/download-artifact@v8": 2,
    "sha256sum -c": 5,
}


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
    path = root / ".github" / "workflows" / "release.yml"
    if not path.exists():
        return [".github/workflows/release.yml: missing release workflow"]

    text = path.read_text(encoding="utf-8")
    errors = [
        f".github/workflows/release.yml: missing {description}"
        for description, snippet in REQUIRED_SNIPPETS.items()
        if snippet not in text
    ]

    for snippet, expected_count in REQUIRED_COUNTS.items():
        actual_count = text.count(snippet)
        if actual_count < expected_count:
            errors.append(
                ".github/workflows/release.yml: "
                f"expected at least {expected_count} occurrence(s) of `{snippet}`, "
                f"found {actual_count}"
            )

    return errors


if __name__ == "__main__":
    raise SystemExit(main())
