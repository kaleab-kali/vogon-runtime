#!/usr/bin/env python3
"""Validate a release SHA-256 checksum file against its artifact."""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path


SHA256_HEX = re.compile(r"^[0-9a-fA-F]{64}$")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=Path, help="Artifact file to verify.")
    parser.add_argument(
        "checksum_file",
        nargs="?",
        type=Path,
        help="Checksum file. Defaults to ARTIFACT.sha256.",
    )
    args = parser.parse_args()

    errors = check_file(args.artifact, args.checksum_file)
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_file(artifact: Path, checksum_file: Path | None = None) -> list[str]:
    checksum_file = checksum_file or Path(f"{artifact}.sha256")

    try:
        artifact_bytes = artifact.read_bytes()
    except OSError as error:
        return [f"Artifact cannot be read: {error}"]

    try:
        checksum_output = checksum_file.read_text(encoding="utf-8-sig")
    except OSError as error:
        return [f"Checksum file cannot be read: {error}"]

    return check_output(
        artifact_name=artifact.name,
        artifact_bytes=artifact_bytes,
        checksum_output=checksum_output,
    )


def check_output(
    *,
    artifact_name: str,
    artifact_bytes: bytes,
    checksum_output: str,
) -> list[str]:
    lines = checksum_output.splitlines()
    if len(lines) != 1 or not lines[0].strip():
        return ["Checksum file must contain exactly one checksum line"]

    line = lines[0]
    fields = line.split(maxsplit=1)
    if len(fields) != 2:
        return ["Checksum line must contain a SHA-256 digest and artifact filename"]

    digest, recorded_name = fields
    if recorded_name.startswith("*"):
        recorded_name = recorded_name[1:]

    errors: list[str] = []
    if SHA256_HEX.fullmatch(digest) is None:
        errors.append("Checksum digest must be 64 hexadecimal characters")

    if recorded_name != artifact_name:
        errors.append(
            f"Checksum filename mismatch: expected {artifact_name}, got {recorded_name}"
        )

    actual_digest = hashlib.sha256(artifact_bytes).hexdigest()
    if digest.lower() != actual_digest:
        errors.append(
            f"Checksum digest mismatch: expected {digest.lower()}, got {actual_digest}"
        )

    return errors


if __name__ == "__main__":
    sys.exit(main())
