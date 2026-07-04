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
          python3 scripts/check_doctor_json.py
          python3 scripts/check_cache_json.py
          python3 scripts/check_workflow_json.py
          python3 scripts/check_verify_json.py
          python3 scripts/check_trace_jsonl.py
          sha256sum -c vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256
          sha256sum -c vogon-${{ github.ref_name }}-cargo-metadata.json.sha256
          sha256sum -c vogon-${{ github.ref_name }}-cargo-spdx.json.sha256
          echo vogon-${{ github.ref_name }}-linux-x86_64.tar.gz
          echo vogon-${{ github.ref_name }}-linux-x86_64.tar.gz.sha256
          echo vogon-${{ github.ref_name }}-cargo-metadata.json.sha256
          echo vogon-${{ github.ref_name }}-cargo-spdx.json.sha256
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
  windows-cli:
    steps:
      - uses: actions/checkout@v7
      - run: |
          sha256sum -c vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256
          echo vogon-${{ github.ref_name }}-windows-x86_64.zip
          echo vogon-${{ github.ref_name }}-windows-x86_64.zip.sha256
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
  container-image:
    steps:
      - uses: actions/checkout@v7
      - run: |
          sha256sum -c vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          sha256sum -c vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          echo vogon-${{ github.ref_name }}-container-image.tar.gz
          echo vogon-${{ github.ref_name }}-container-image.tar.gz.sha256
          echo 'index .Config.Labels "org.opencontainers.image.source"'
          echo 'index .Config.Labels "org.opencontainers.image.licenses"'
          docker run --rm --entrypoint id "$image" -u
          docker run --rm --read-only "$image" --version
      - uses: actions/attest@v4
      - uses: actions/upload-artifact@v7
        with:
          if-no-files-found: error
  release-artifact-smoke:
    steps:
      - uses: actions/checkout@v7
      - uses: actions/download-artifact@v8
  publish-release:
    steps:
      - uses: actions/download-artifact@v8
      - run: gh release create "${{ github.ref_name }}"
"""


if __name__ == "__main__":
    unittest.main()
