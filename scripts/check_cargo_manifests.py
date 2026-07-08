#!/usr/bin/env python3
"""Validate Cargo manifest metadata for open-source package readiness."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path
from typing import Any


EXPECTED_WORKSPACE_PACKAGE = {
    "edition": "2024",
    "rust-version": "1.85",
    "license": "MIT",
    "repository": "https://github.com/kaleab-kali/vogon-runtime",
    "homepage": "https://github.com/kaleab-kali/vogon-runtime",
    "documentation": "https://github.com/kaleab-kali/vogon-runtime/tree/main/docs",
    "authors": ["Vogon Runtime Contributors"],
}
REQUIRED_PACKAGE_FIELDS = {
    "name",
    "version",
    "edition",
    "rust-version",
    "license",
    "repository",
    "homepage",
    "documentation",
    "authors",
    "description",
    "readme",
    "keywords",
    "categories",
}
EXPECTED_CRATES = {
    "vogon-adapters": "crates/vogon-adapters",
    "vogon-cli": "crates/vogon-cli",
    "vogon-core": "crates/vogon-core",
    "vogon-xtask": "crates/vogon-xtask",
}
EXPECTED_RELEASE_PROFILE = {
    "lto": "thin",
    "codegen-units": 1,
    "strip": "symbols",
}
EXPECTED_WORKSPACE_RUST_LINTS = {
    "unsafe_code": "forbid",
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
    workspace_path = root / "Cargo.toml"
    if not workspace_path.exists():
        return ["Cargo.toml: missing workspace manifest"]

    errors: list[str] = []
    workspace = read_manifest(workspace_path)
    workspace_package = workspace.get("workspace", {}).get("package")
    if not isinstance(workspace_package, dict):
        errors.append("Cargo.toml: missing [workspace.package]")
        workspace_package = {}
    errors.extend(check_workspace_package(workspace_package))

    members = workspace.get("workspace", {}).get("members")
    if members != sorted(EXPECTED_CRATES.values()):
        errors.append(
            "Cargo.toml: workspace members must be "
            + ", ".join(sorted(EXPECTED_CRATES.values()))
        )

    release_profile = workspace.get("profile", {}).get("release")
    if not isinstance(release_profile, dict):
        errors.append("Cargo.toml: missing [profile.release]")
        release_profile = {}
    errors.extend(check_release_profile(release_profile))

    workspace_rust_lints = workspace.get("workspace", {}).get("lints", {}).get("rust")
    if not isinstance(workspace_rust_lints, dict):
        errors.append("Cargo.toml: missing [workspace.lints.rust]")
        workspace_rust_lints = {}
    errors.extend(check_workspace_rust_lints(workspace_rust_lints))

    workspace_deps = workspace.get("workspace", {}).get("dependencies", {})
    if not isinstance(workspace_deps, dict):
        errors.append("Cargo.toml: missing [workspace.dependencies]")
        workspace_deps = {}

    crate_versions: dict[str, str] = {}
    for crate_name, crate_dir in sorted(EXPECTED_CRATES.items()):
        manifest_path = root / crate_dir / "Cargo.toml"
        if not manifest_path.exists():
            errors.append(f"{crate_dir}/Cargo.toml: missing crate manifest")
            continue
        manifest = read_manifest(manifest_path)
        package = manifest.get("package")
        if not isinstance(package, dict):
            errors.append(f"{crate_dir}/Cargo.toml: missing [package]")
            continue
        errors.extend(check_crate_package(root, manifest_path, crate_name, package))
        errors.extend(check_crate_lints(root, manifest_path, manifest))
        version = package.get("version")
        if isinstance(version, str):
            crate_versions[crate_name] = version

    versions = set(crate_versions.values())
    if len(versions) > 1:
        errors.append("Cargo.toml: workspace crate versions must match")

    for crate_name in ("vogon-adapters", "vogon-core"):
        dependency = workspace_deps.get(crate_name)
        dependency_version = dependency.get("version") if isinstance(dependency, dict) else None
        crate_version = crate_versions.get(crate_name)
        if crate_version and dependency_version != crate_version:
            errors.append(
                f"Cargo.toml: workspace dependency `{crate_name}` version must match crate version {crate_version}"
            )

    return errors


def read_manifest(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def check_workspace_package(package: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for key, expected in EXPECTED_WORKSPACE_PACKAGE.items():
        if package.get(key) != expected:
            errors.append(f"Cargo.toml: workspace package `{key}` must be {expected!r}")
    return errors


def check_release_profile(profile: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for key, expected in EXPECTED_RELEASE_PROFILE.items():
        if profile.get(key) != expected:
            errors.append(f"Cargo.toml: release profile `{key}` must be {expected!r}")
    return errors


def check_workspace_rust_lints(lints: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for key, expected in EXPECTED_WORKSPACE_RUST_LINTS.items():
        if lints.get(key) != expected:
            errors.append(f"Cargo.toml: workspace rust lint `{key}` must be {expected!r}")
    return errors


def check_crate_package(
    root: Path,
    manifest_path: Path,
    expected_name: str,
    package: dict[str, Any],
) -> list[str]:
    relative_path = manifest_path.relative_to(root).as_posix()
    errors: list[str] = []

    for field in sorted(REQUIRED_PACKAGE_FIELDS):
        if field not in package:
            errors.append(f"{relative_path}: package missing `{field}`")

    if package.get("name") != expected_name:
        errors.append(f"{relative_path}: package name must be `{expected_name}`")

    for workspace_field in EXPECTED_WORKSPACE_PACKAGE:
        value = package.get(workspace_field)
        if not (isinstance(value, dict) and value.get("workspace") is True):
            errors.append(f"{relative_path}: package `{workspace_field}` must use workspace metadata")

    readme = package.get("readme")
    if isinstance(readme, str):
        readme_path = (manifest_path.parent / readme).resolve()
        if not readme_path.exists():
            errors.append(f"{relative_path}: readme path `{readme}` does not exist")

    for list_field in ("keywords", "categories"):
        value = package.get(list_field)
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            errors.append(f"{relative_path}: package `{list_field}` must be a string list")
        elif not value:
            errors.append(f"{relative_path}: package `{list_field}` must not be empty")

    description = package.get("description")
    if not isinstance(description, str) or not description.strip():
        errors.append(f"{relative_path}: package `description` must not be empty")

    return errors


def check_crate_lints(root: Path, manifest_path: Path, manifest: dict[str, Any]) -> list[str]:
    relative_path = manifest_path.relative_to(root).as_posix()
    lints = manifest.get("lints")
    if not (isinstance(lints, dict) and lints.get("workspace") is True):
        return [f"{relative_path}: crate lints must use workspace policy"]
    return []


if __name__ == "__main__":
    raise SystemExit(main())
