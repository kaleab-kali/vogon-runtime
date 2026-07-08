#!/usr/bin/env python3
"""Validate container build hardening conventions."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


FROM_RE = re.compile(r"^FROM\s+([^\s]+)(?:\s+AS\s+([A-Za-z0-9_-]+))?\s*$", re.I)
REQUIRED_DOCKERIGNORE_ENTRIES = {
    "/.git",
    "/.github",
    "/target",
    ".env",
    ".env.*",
    "!.env.example",
    "__pycache__/",
    "*.py[cod]",
    "*.cache.json",
}
REQUIRED_DOCKERFILE_SNIPPETS = {
    "cargo incremental builds disabled": "ENV CARGO_INCREMENTAL=0",
    "cargo network retries configured": "ENV CARGO_NET_RETRY=10",
    "runtime stage": "FROM debian:bookworm-slim AS runtime",
    "minimal certificate install": (
        "apt-get install -y --no-install-recommends ca-certificates"
    ),
    "OCI title label": 'org.opencontainers.image.title="Vogon Runtime"',
    "OCI description label": (
        'org.opencontainers.image.description="Deterministic, replayable AI workflow runtime CLI."'
    ),
    "OCI source label": (
        'org.opencontainers.image.source="https://github.com/kaleab-kali/vogon-runtime"'
    ),
    "OCI documentation label": (
        'org.opencontainers.image.documentation="https://github.com/kaleab-kali/vogon-runtime#readme"'
    ),
    "OCI license label": 'org.opencontainers.image.licenses="MIT"',
    "apt package list cleanup": "rm -rf /var/lib/apt/lists/*",
    "non-root runtime user": "useradd --create-home --uid 10001 vogon",
    "release binary copy": (
        "COPY --from=build /workspace/target/release/vogon /usr/local/bin/vogon"
    ),
    "non-root user activation": "USER vogon",
    "runtime workdir": "WORKDIR /work",
    "exec entrypoint": 'ENTRYPOINT ["vogon"]',
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
    errors: list[str] = []
    errors.extend(check_dockerfile(root))
    errors.extend(check_dockerignore(root))
    return errors


def check_dockerfile(root: Path) -> list[str]:
    path = root / "Dockerfile"
    if not path.exists():
        return ["Dockerfile: missing container build file"]

    lines = path.read_text(encoding="utf-8").splitlines()
    text = "\n".join(lines)
    errors: list[str] = []

    for description, snippet in REQUIRED_DOCKERFILE_SNIPPETS.items():
        if snippet not in text:
            errors.append(f"Dockerfile: missing {description}")

    for line_number, line in enumerate(lines, start=1):
        match = FROM_RE.match(line.strip())
        if match:
            image = match.group(1)
            if image_reference_uses_latest(image):
                errors.append(
                    f"Dockerfile:{line_number}: base image `{image}` must not use latest"
                )
            if ":" not in image and "@" not in image:
                errors.append(
                    f"Dockerfile:{line_number}: base image `{image}` must include a tag or digest"
                )

    return errors


def image_reference_uses_latest(image: str) -> bool:
    if "@" in image:
        image = image.split("@", 1)[0]
    last_path_segment = image.rsplit("/", 1)[-1]
    if ":" not in last_path_segment:
        return False
    return last_path_segment.rsplit(":", 1)[1] == "latest"


def check_dockerignore(root: Path) -> list[str]:
    path = root / ".dockerignore"
    if not path.exists():
        return [".dockerignore: missing container build context ignore file"]

    entries = {
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    }
    return [
        f".dockerignore: missing {entry}"
        for entry in sorted(REQUIRED_DOCKERIGNORE_ENTRIES - entries)
    ]


if __name__ == "__main__":
    raise SystemExit(main())
