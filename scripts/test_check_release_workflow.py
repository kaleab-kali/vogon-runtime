import tempfile
import unittest
from pathlib import Path

from scripts import check_release_workflow


class CheckReleaseWorkflowTests(unittest.TestCase):
    def test_accepts_release_workflow_contract(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_release_workflow(root, release_workflow_text())

            self.assertEqual(check_release_workflow.check_repository(root), [])

    def test_reports_missing_release_workflow(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_release_workflow.check_repository(Path(directory))

            self.assertEqual(
                errors,
                [".github/workflows/release.yml: missing release workflow"],
            )

    def test_reports_missing_required_snippet(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_release_workflow(
                root,
                release_workflow_text().replace(
                    "python3 scripts/write_spdx_sbom.py",
                    "python3 scripts/write_other_sbom.py",
                ),
            )

            errors = check_release_workflow.check_repository(root)

            self.assertIn(
                ".github/workflows/release.yml: missing SPDX SBOM writer",
                errors,
            )

    def test_reports_missing_required_occurrence_count(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_release_workflow(
                root,
                release_workflow_text().replace(
                    "uses: actions/attest@v4",
                    "uses: actions/attest@v4",
                    2,
                ).replace("uses: actions/attest@v4", "uses: actions/checkout@v7", 1),
            )

            errors = check_release_workflow.check_repository(root)

            self.assertIn(
                ".github/workflows/release.yml: expected at least 3 occurrence(s) of `uses: actions/attest@v4`, found 2",
                errors,
            )

    def test_reports_missing_artifact_retention_count(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_release_workflow(
                root,
                release_workflow_text().replace(
                    "          retention-days: 30\n",
                    "",
                    1,
                ),
            )

            errors = check_release_workflow.check_repository(root)

            self.assertIn(
                ".github/workflows/release.yml: expected at least 3 occurrence(s) of `retention-days: 30`, found 2",
                errors,
            )


def write_release_workflow(root: Path, text: str) -> None:
    workflows = root / ".github" / "workflows"
    workflows.mkdir(parents=True)
    (workflows / "release.yml").write_text(text, encoding="utf-8")


def release_workflow_text() -> str:
    return """name: Release

on:
  push:
    tags:
      - "v*.*.*"
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false

jobs:
  linux-cli:
    steps:
      - uses: actions/checkout@v7
      - run: cargo build --release -p vogon-cli --locked
      - run: |
          cargo metadata --locked --format-version 1
          python3 scripts/check_cargo_metadata_json.py
          python3 scripts/write_spdx_sbom.py
          python3 scripts/check_spdx_sbom_json.py
          python3 scripts/check_sha256_file.py
          python3 scripts/check_archive_contents.py
          python3 scripts/check_sha256_file.py
          python3 scripts/check_sha256_file.py
          python3 scripts/check_doctor_json.py
          python3 scripts/check_cache_json.py
          python3 scripts/check_workflow_json.py
          python3 scripts/check_verify_json.py
          python3 scripts/check_trace_jsonl.py
          sha256sum -c vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256
          sha256sum -c vogon-${{ github.ref_name }}-cargo-metadata.json.sha256
          sha256sum -c vogon-${{ github.ref_name }}-cargo-spdx.json.sha256
          python3 scripts/check_sha256_file.py
          python3 scripts/check_sha256_file.py
          python3 scripts/check_sha256_file.py
          tar -xzf "vogon-${{ github.ref_name }}-linux-x86_64.tar.gz" -C archive-smoke
          python3 scripts/check_archive_contents.py archive-smoke --binary vogon
          ./archive-smoke/vogon --version
          echo vogon-${{ github.ref_name }}-linux-x86_64.tar.gz
          echo vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256
          echo vogon-${{ github.ref_name }}-cargo-metadata.json.sha256
          echo vogon-${{ github.ref_name }}-cargo-spdx.json.sha256
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
          retention-days: 30
  windows-cli:
    steps:
      - uses: actions/checkout@v7
      - run: |
          python3 scripts/check_sha256_file.py
          python3 scripts/check_archive_contents.py
          sha256sum -c vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256
          python3 scripts/check_sha256_file.py
          Expand-Archive "vogon-${{ github.ref_name }}-windows-x86_64.zip" -DestinationPath archive-smoke -Force
          python scripts/check_archive_contents.py archive-smoke --binary vogon.exe
          .\\archive-smoke\\vogon.exe --version
          echo vogon-${{ github.ref_name }}-windows-x86_64.zip
          echo vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
          retention-days: 30
  container-image:
    steps:
      - uses: actions/checkout@v7
      - run: |
          python3 scripts/check_sha256_file.py
          sha256sum -c vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          sha256sum -c vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          python3 scripts/check_sha256_file.py
          echo vogon-${{ github.ref_name }}-container-image.tar.gz
          echo vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          python3 scripts/check_container_image.py
          python3 scripts/check_container_image.py
          python3 scripts/check_container_image.py
          docker run --rm --read-only "$image" --version
          docker run --rm --read-only "$image" doctor --json | python3 "$GITHUB_WORKSPACE/scripts/check_doctor_json.py"
          docker run --rm --read-only -v "${{ runner.temp }}/vogon-downloaded-container-smoke:/work:ro" "$image" check --json /work/starter.toml | python3 "$GITHUB_WORKSPACE/scripts/check_workflow_json.py"
          python3 "$GITHUB_WORKSPACE/scripts/check_cache_json.py" "${{ runner.temp }}/vogon-downloaded-container-smoke/cache-smoke.cache.json"
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
          retention-days: 30
  release-artifact-smoke:
    steps:
      - uses: actions/checkout@v7
      - uses: actions/download-artifact@v8
      - run: |
          python3 scripts/check_archive_contents.py
          python3 scripts/check_archive_contents.py
  publish-release:
    steps:
      - uses: actions/download-artifact@v8
      - run: gh release create "${{ github.ref_name }}"
"""


if __name__ == "__main__":
    unittest.main()
