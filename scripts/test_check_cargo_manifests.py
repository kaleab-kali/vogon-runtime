import tempfile
import unittest
from pathlib import Path

from scripts import check_cargo_manifests


class CheckCargoManifestsTests(unittest.TestCase):
    def test_accepts_valid_workspace_manifests(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root)

            self.assertEqual(check_cargo_manifests.check_repository(root), [])

    def test_reports_missing_workspace_package_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root, workspace_package="edition = \"2024\"\n")

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn("Cargo.toml: workspace package `license` must be 'MIT'", errors)
            self.assertIn(
                "Cargo.toml: workspace package `repository` must be 'https://github.com/kaleab-kali/vogon-runtime'",
                errors,
            )

    def test_reports_missing_crate_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root)
            manifest = root / "crates" / "vogon-core" / "Cargo.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    "description = \"Core deterministic workflow runtime for Vogon Runtime.\"\n",
                    "",
                ),
                encoding="utf-8",
            )

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn(
                "crates/vogon-core/Cargo.toml: package missing `description`",
                errors,
            )
            self.assertIn(
                "crates/vogon-core/Cargo.toml: package `description` must not be empty",
                errors,
            )

    def test_reports_internal_dependency_version_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root, adapters_dependency_version="9.9.9")

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn(
                "Cargo.toml: workspace dependency `vogon-adapters` version must match crate version 0.1.0",
                errors,
            )

    def test_reports_weakened_release_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(
                root,
                release_profile=release_profile_text().replace(
                    "lto = \"thin\"",
                    "lto = false",
                ),
            )

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn(
                "Cargo.toml: release profile `lto` must be 'thin'",
                errors,
            )

    def test_reports_missing_workspace_unsafe_lint(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root, workspace_lints="")

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn("Cargo.toml: missing [workspace.lints.rust]", errors)
            self.assertIn(
                "Cargo.toml: workspace rust lint `unsafe_code` must be 'forbid'",
                errors,
            )

    def test_reports_crate_that_does_not_use_workspace_lints(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_workspace(root, crate_lints="")

            errors = check_cargo_manifests.check_repository(root)

            self.assertIn(
                "crates/vogon-core/Cargo.toml: crate lints must use workspace policy",
                errors,
            )


def write_workspace(
    root: Path,
    *,
    workspace_package: str | None = None,
    adapters_dependency_version: str = "0.1.0",
    release_profile: str | None = None,
    workspace_lints: str | None = None,
    crate_lints: str | None = None,
) -> None:
    (root / "README.md").write_text("# Vogon Runtime\n", encoding="utf-8")
    (root / "Cargo.toml").write_text(
        """[workspace]
resolver = "3"
members = [
    "crates/vogon-adapters",
    "crates/vogon-cli",
    "crates/vogon-core",
]

[workspace.package]
"""
        + (workspace_package or workspace_package_text())
        + f"""
[workspace.dependencies]
vogon-adapters = {{ version = "{adapters_dependency_version}", path = "crates/vogon-adapters" }}
vogon-core = {{ version = "0.1.0", path = "crates/vogon-core" }}
"""
        + (workspace_lints if workspace_lints is not None else workspace_lints_text())
        + (release_profile or release_profile_text()),
        encoding="utf-8",
    )
    write_crate_manifest(
        root,
        "vogon-core",
        "Core deterministic workflow runtime for Vogon Runtime.",
        ["ai", "workflow", "replay", "runtime"],
        ["development-tools"],
        crate_lints=crate_lints,
    )
    write_crate_manifest(
        root,
        "vogon-adapters",
        "Model adapters for Vogon Runtime.",
        ["ai", "model-adapters", "workflow", "runtime"],
        ["development-tools"],
        crate_lints=crate_lints,
    )
    write_crate_manifest(
        root,
        "vogon-cli",
        "Command-line interface for Vogon Runtime.",
        ["ai", "workflow", "replay", "cli"],
        ["command-line-utilities", "development-tools"],
        crate_lints=crate_lints,
    )


def workspace_package_text() -> str:
    return """edition = "2024"
rust-version = "1.85"
license = "MIT"
repository = "https://github.com/kaleab-kali/vogon-runtime"
homepage = "https://github.com/kaleab-kali/vogon-runtime"
documentation = "https://github.com/kaleab-kali/vogon-runtime/tree/main/docs"
authors = ["Vogon Runtime Contributors"]
"""


def release_profile_text() -> str:
    return """
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
"""


def workspace_lints_text() -> str:
    return """
[workspace.lints.rust]
unsafe_code = "forbid"
"""


def write_crate_manifest(
    root: Path,
    name: str,
    description: str,
    keywords: list[str],
    categories: list[str],
    *,
    crate_lints: str | None = None,
) -> None:
    crate_dir = root / "crates" / name
    crate_dir.mkdir(parents=True)
    (crate_dir / "Cargo.toml").write_text(
        f"""[package]
name = "{name}"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
authors.workspace = true
description = "{description}"
readme = "../../README.md"
keywords = {keywords!r}
categories = {categories!r}
"""
        + (crate_lints if crate_lints is not None else crate_lints_text()),
        encoding="utf-8",
    )


def crate_lints_text() -> str:
    return """
[lints]
workspace = true
"""
