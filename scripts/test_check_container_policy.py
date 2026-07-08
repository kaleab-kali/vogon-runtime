import tempfile
import unittest
from pathlib import Path

from scripts import check_container_policy


class CheckContainerPolicyTests(unittest.TestCase):
    def test_accepts_hardened_container_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_container_files(root)

            self.assertEqual(check_container_policy.check_repository(root), [])

    def test_reports_missing_container_files(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_container_policy.check_repository(Path(directory))

            self.assertEqual(
                errors,
                [
                    "Dockerfile: missing container build file",
                    ".dockerignore: missing container build context ignore file",
                ],
            )

    def test_rejects_latest_and_untagged_base_images(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_container_files(root)
            dockerfile = root / "Dockerfile"
            dockerfile.write_text(
                dockerfile.read_text(encoding="utf-8")
                .replace("rust:1.85.0-bookworm", "rust")
                .replace("debian:bookworm-slim", "debian:latest"),
                encoding="utf-8",
            )

            errors = check_container_policy.check_repository(root)

            self.assertIn(
                "Dockerfile:3: base image `rust` must include a tag or digest",
                errors,
            )
            self.assertIn(
                "Dockerfile:15: base image `debian:latest` must not use latest",
                errors,
            )

    def test_reports_missing_runtime_hardening(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_container_files(root)
            dockerfile = root / "Dockerfile"
            dockerfile.write_text(
                dockerfile.read_text(encoding="utf-8").replace("USER vogon", ""),
                encoding="utf-8",
            )

            errors = check_container_policy.check_repository(root)

            self.assertEqual(errors, ["Dockerfile: missing non-root user activation"])

    def test_reports_missing_oci_metadata_label(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_container_files(root)
            dockerfile = root / "Dockerfile"
            dockerfile.write_text(
                dockerfile.read_text(encoding="utf-8").replace(
                    '    org.opencontainers.image.licenses="MIT" \\\n',
                    "",
                ),
                encoding="utf-8",
            )

            errors = check_container_policy.check_repository(root)

            self.assertEqual(errors, ["Dockerfile: missing OCI license label"])

    def test_reports_missing_build_context_ignores(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_container_files(root, dockerignore="/.git\n")

            errors = check_container_policy.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".dockerignore: missing !.env.example",
                    ".dockerignore: missing *.cache.json",
                    ".dockerignore: missing *.py[cod]",
                    ".dockerignore: missing .env",
                    ".dockerignore: missing .env.*",
                    ".dockerignore: missing /.github",
                    ".dockerignore: missing /target",
                    ".dockerignore: missing __pycache__/",
                ],
            )


def write_container_files(root: Path, *, dockerignore: str | None = None) -> None:
    (root / "Dockerfile").write_text(
        "\n".join(
            [
                "# syntax=docker/dockerfile:1",
                "",
                "FROM rust:1.85.0-bookworm AS build",
                "",
                "WORKDIR /workspace",
                "",
                "ENV CARGO_INCREMENTAL=0",
                "ENV CARGO_NET_RETRY=10",
                "",
                "COPY Cargo.toml Cargo.lock rust-toolchain.toml ./",
                "COPY crates ./crates",
                "",
                "RUN cargo build --release --locked -p vogon-cli",
                "",
                "FROM debian:bookworm-slim AS runtime",
                "",
                "ARG VOGON_IMAGE_VERSION=dev",
                "ARG VOGON_IMAGE_REVISION=unknown",
                "",
                'LABEL org.opencontainers.image.title="Vogon Runtime" \\',
                '    org.opencontainers.image.description="Deterministic, replayable AI workflow runtime CLI." \\',
                '    org.opencontainers.image.source="https://github.com/kaleab-kali/vogon-runtime" \\',
                '    org.opencontainers.image.documentation="https://github.com/kaleab-kali/vogon-runtime#readme" \\',
                '    org.opencontainers.image.licenses="MIT" \\',
                '    org.opencontainers.image.version="${VOGON_IMAGE_VERSION}" \\',
                '    org.opencontainers.image.revision="${VOGON_IMAGE_REVISION}"',
                "",
                "RUN apt-get update \\",
                "    && apt-get install -y --no-install-recommends ca-certificates \\",
                "    && rm -rf /var/lib/apt/lists/* \\",
                "    && useradd --create-home --uid 10001 vogon",
                "",
                "COPY --from=build /workspace/target/release/vogon /usr/local/bin/vogon",
                "",
                "USER vogon",
                "WORKDIR /work",
                'ENTRYPOINT ["vogon"]',
            ]
        ),
        encoding="utf-8",
    )
    (root / ".dockerignore").write_text(
        dockerignore
        or (
            "/.git\n"
            "/.github\n"
            "/target\n"
            ".env\n"
            ".env.*\n"
            "!.env.example\n"
            "__pycache__/\n"
            "*.py[cod]\n"
            "*.cache.json\n"
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
