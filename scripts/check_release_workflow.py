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
    "concurrency group": "concurrency:\n  group: ${{ github.workflow }}-${{ github.ref }}",
    "no release cancellation": "  cancel-in-progress: false",
    "linux artifact job": "  linux-cli:",
    "windows artifact job": "  windows-cli:",
    "container artifact job": "  container-image:",
    "release artifact download smoke job": "  release-artifact-smoke:",
    "publish release job": "  publish-release:",
    "linux release build": "cargo build --release -p vogon-cli --locked",
    "linux archive": "vogon-${{ github.ref_name }}-linux-x86_64.tar.gz",
    "windows archive": "vogon-${{ github.ref_name }}-windows-x86_64.zip",
    "container archive": "vogon-${{ github.ref_name }}-container-image.tar.gz",
    "container version build argument": (
        '--build-arg "VOGON_IMAGE_VERSION=${{ github.ref_name }}"'
    ),
    "container revision build argument": (
        '--build-arg "VOGON_IMAGE_REVISION=${{ github.sha }}"'
    ),
    "dependency metadata": "cargo metadata --locked --format-version 1",
    "dependency metadata validator": "check-cargo-metadata-json",
    "SPDX SBOM writer": "python3 scripts/write_spdx_sbom.py",
    "SPDX SBOM validator": "scripts/check_spdx_sbom_json.py",
    "SHA-256 checksum validator": "check-sha256-file",
    "archive contents validator": "check-archive-contents",
    "linux archive contents before smoke outputs": (
        "tar -xzf \"vogon-${{ github.ref_name }}-linux-x86_64.tar.gz\" -C archive-smoke\n"
        "          cargo run -p vogon-xtask -- check-archive-contents archive-smoke --binary vogon\n"
        "          ./archive-smoke/vogon --version"
    ),
    "windows archive contents before smoke outputs": (
        "Expand-Archive \"vogon-${{ github.ref_name }}-windows-x86_64.zip\" -DestinationPath archive-smoke -Force\n"
        "          cargo run -p vogon-xtask -- check-archive-contents archive-smoke --binary vogon.exe\n"
        "          .\\archive-smoke\\vogon.exe --version"
    ),
    "doctor JSON validator": "scripts/check_doctor_json.py",
    "providers JSON validator": "scripts/check_providers_json.py",
    "cache JSON validator": "scripts/check_cache_json.py",
    "workflow check JSON validator": "scripts/check_workflow_json.py",
    "verify JSON validator": "scripts/check_verify_json.py",
    "trace JSONL validator": "scripts/check_trace_jsonl.py",
    "container image validator": "scripts/check_container_image.py",
    "container version label validation": '--expected-version "${{ github.ref_name }}"',
    "container revision label validation": '--expected-revision "${{ github.sha }}"',
    "Linux checksum": "vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256",
    "Windows checksum": "vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256",
    "container checksum": "vogon-${{ github.ref_name }}-container-image.tar.gz.sha256",
    "metadata checksum": "vogon-${{ github.ref_name }}-cargo-metadata.json.sha256",
    "SBOM checksum": "vogon-${{ github.ref_name }}-cargo-spdx.json.sha256",
    "artifact attestation": "uses: actions/attest@v4",
    "read-only release job contents permission": "      contents: read",
    "release attestation OIDC permission": "      id-token: write",
    "release attestation write permission": "      attestations: write",
    "artifact upload": "uses: actions/upload-artifact@v7",
    "artifact download": "uses: actions/download-artifact@v8",
    "publish release checkout": (
        "  publish-release:\n"
        "    name: Publish GitHub release\n"
        "    if: github.ref_type == 'tag'\n"
        "    runs-on: ubuntu-24.04\n"
        "    timeout-minutes: 10\n"
        "    permissions:\n"
        "      contents: write\n"
        "    needs:\n"
        "      - linux-cli\n"
        "      - windows-cli\n"
        "      - container-image\n"
        "\n"
        "    steps:\n"
        "      - name: Checkout\n"
        "        uses: actions/checkout@v7\n"
        "        with:\n"
        "          persist-credentials: false"
    ),
    "missing artifact failure": "if-no-files-found: error",
    "artifact retention": "retention-days: 30",
    "GitHub release creation": "gh release create",
    "read-only container smoke": "docker run --rm --read-only",
    "downloaded container doctor validator": 'python3 "$GITHUB_WORKSPACE/scripts/check_doctor_json.py"',
    "downloaded container providers validator": (
        'python3 "$GITHUB_WORKSPACE/scripts/check_providers_json.py"'
    ),
    "downloaded container workflow validator": 'python3 "$GITHUB_WORKSPACE/scripts/check_workflow_json.py"',
    "downloaded container cache validator": 'python3 "$GITHUB_WORKSPACE/scripts/check_cache_json.py"',
}

REQUIRED_COUNTS = {
    "uses: actions/checkout@v7": 5,
    "uses: actions/attest@v4": 3,
    "      id-token: write": 3,
    "      attestations: write": 3,
    "uses: actions/upload-artifact@v7": 3,
    "uses: actions/download-artifact@v8": 2,
    "retention-days: 30": 3,
    "sha256sum -c": 5,
    "check-sha256-file": 10,
    "check-archive-contents": 4,
    "scripts/check_providers_json.py": 5,
    "scripts/check_container_image.py": 3,
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
