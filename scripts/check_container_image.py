#!/usr/bin/env python3
"""Validate runtime metadata and hardening on a built container image."""

from __future__ import annotations

import argparse
import subprocess
import sys
from collections.abc import Callable, Sequence


EXPECTED_LABELS = {
    "org.opencontainers.image.title": "Vogon Runtime",
    "org.opencontainers.image.source": "https://github.com/kaleab-kali/vogon-runtime",
    "org.opencontainers.image.licenses": "MIT",
    "org.opencontainers.image.version": "dev",
    "org.opencontainers.image.revision": "unknown",
}
EXPECTED_USER_ID = "10001"

Runner = Callable[[Sequence[str]], subprocess.CompletedProcess[str]]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", help="Container image reference to validate.")
    parser.add_argument(
        "--expected-user-id",
        default=EXPECTED_USER_ID,
        help=f"Expected runtime user id. Defaults to {EXPECTED_USER_ID}.",
    )
    parser.add_argument(
        "--expected-version",
        default=EXPECTED_LABELS["org.opencontainers.image.version"],
        help="Expected org.opencontainers.image.version label.",
    )
    parser.add_argument(
        "--expected-revision",
        default=EXPECTED_LABELS["org.opencontainers.image.revision"],
        help="Expected org.opencontainers.image.revision label.",
    )
    args = parser.parse_args()

    expected_labels = {
        **EXPECTED_LABELS,
        "org.opencontainers.image.version": args.expected_version,
        "org.opencontainers.image.revision": args.expected_revision,
    }
    errors = check_image(
        args.image,
        expected_labels=expected_labels,
        expected_user_id=args.expected_user_id,
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_image(
    image: str,
    *,
    expected_labels: dict[str, str] | None = None,
    expected_user_id: str = EXPECTED_USER_ID,
    runner: Runner | None = None,
) -> list[str]:
    runner = runner or run_command
    expected_labels = expected_labels or EXPECTED_LABELS
    errors: list[str] = []

    for label, expected_value in expected_labels.items():
        result = runner(
            [
                "docker",
                "image",
                "inspect",
                image,
                "--format",
                f'{{{{ index .Config.Labels "{label}" }}}}',
            ]
        )
        if result.returncode != 0:
            errors.append(
                format_command_error(f"Container label {label} cannot be read", result)
            )
            continue

        actual_value = result.stdout.strip()
        if actual_value != expected_value:
            errors.append(
                f"Container label {label} mismatch: "
                f"expected {expected_value}, got {actual_value or '<empty>'}"
            )

    result = runner(["docker", "run", "--rm", "--entrypoint", "id", image, "-u"])
    if result.returncode != 0:
        errors.append(
            format_command_error("Container runtime user cannot be read", result)
        )
    else:
        actual_user_id = result.stdout.strip()
        if actual_user_id != expected_user_id:
            errors.append(
                f"Container runtime user mismatch: "
                f"expected {expected_user_id}, got {actual_user_id or '<empty>'}"
            )

    return errors


def run_command(command: Sequence[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, check=False, text=True)


def format_command_error(context: str, result: subprocess.CompletedProcess[str]) -> str:
    stderr = result.stderr.strip()
    if stderr:
        return f"{context}: {stderr}"
    return f"{context}: command exited with status {result.returncode}"


if __name__ == "__main__":
    sys.exit(main())
