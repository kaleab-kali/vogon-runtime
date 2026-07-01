#!/usr/bin/env python3
"""Validate public GitHub issue templates for open-source readiness."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


ISSUE_TEMPLATE_DIR = Path(".github") / "ISSUE_TEMPLATE"
BUG_REQUIRED_FIELDS = {
    "version",
    "component",
    "expected",
    "actual",
    "reproduce",
    "environment",
    "checks",
}
FEATURE_REQUIRED_FIELDS = {
    "problem",
    "proposal",
    "area",
    "checks",
}
REQUIRED_AREAS = {
    "CLI",
    "Runtime",
    "Replay verification",
    "Provider adapter",
    "Documentation",
    "Release artifact",
    "Other",
}
REQUIRED_CHECK_LABELS = {
    "removed secrets",
    "searched existing issues",
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
    template_dir = root / ISSUE_TEMPLATE_DIR
    config_path = template_dir / "config.yml"
    bug_path = template_dir / "bug_report.yml"
    feature_path = template_dir / "feature_request.yml"

    errors.extend(check_config(root, config_path))
    errors.extend(
        check_form(
            root,
            bug_path,
            expected_name="Bug report",
            expected_title='title: "Bug: "',
            expected_label="- bug",
            required_fields=BUG_REQUIRED_FIELDS,
        )
    )
    errors.extend(
        check_form(
            root,
            feature_path,
            expected_name="Feature request",
            expected_title='title: "Feature: "',
            expected_label="- enhancement",
            required_fields=FEATURE_REQUIRED_FIELDS,
        )
    )
    return errors


def check_config(root: Path, path: Path) -> list[str]:
    relative = relative_path(root, path)
    if not path.exists():
        return [f"{relative}: missing issue template config"]

    text = path.read_text(encoding="utf-8")
    errors: list[str] = []
    if "blank_issues_enabled: false" not in text:
        errors.append(f"{relative}: blank issues must stay disabled")
    if "https://github.com/kaleab-kali/vogon-runtime/security/advisories/new" not in text:
        errors.append(f"{relative}: missing private vulnerability reporting link")
    return errors


def check_form(
    root: Path,
    path: Path,
    *,
    expected_name: str,
    expected_title: str,
    expected_label: str,
    required_fields: set[str],
) -> list[str]:
    relative = relative_path(root, path)
    if not path.exists():
        return [f"{relative}: missing issue form"]

    lines = path.read_text(encoding="utf-8").splitlines()
    text = "\n".join(lines)
    errors: list[str] = []

    if f"name: {expected_name}" not in text:
        errors.append(f"{relative}: missing expected name `{expected_name}`")
    if expected_title not in text:
        errors.append(f"{relative}: missing expected title prefix")
    if expected_label not in text:
        errors.append(f"{relative}: missing expected label `{expected_label}`")

    field_ids = field_ids_from_lines(lines)
    for field_id in sorted(required_fields):
        if field_id not in field_ids:
            errors.append(f"{relative}: missing required field `{field_id}`")

    errors.extend(check_dropdown_options(relative, lines))
    errors.extend(check_before_submit(relative, lines))
    return errors


def field_ids_from_lines(lines: list[str]) -> set[str]:
    ids: set[str] = set()
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("id: "):
            ids.add(stripped.removeprefix("id: ").strip())
    return ids


def check_dropdown_options(relative: str, lines: list[str]) -> list[str]:
    options = options_for_dropdown(lines, {"component", "area"})
    if not options:
        return [f"{relative}: missing component or area dropdown options"]
    missing = sorted(REQUIRED_AREAS - options)
    if missing:
        return [f"{relative}: dropdown options missing {', '.join(missing)}"]
    return []


def options_for_dropdown(lines: list[str], field_ids: set[str]) -> set[str]:
    options: set[str] = set()
    in_target_dropdown = False
    in_options = False

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("id: "):
            in_target_dropdown = stripped.removeprefix("id: ").strip() in field_ids
            in_options = False
            continue
        if in_target_dropdown and stripped == "options:":
            in_options = True
            continue
        if in_options and stripped.startswith("- "):
            options.add(stripped.removeprefix("- ").strip())
            continue
        if in_options and stripped and not line.startswith(" " * 8):
            in_options = False

    return options


def check_before_submit(relative: str, lines: list[str]) -> list[str]:
    check_labels = [
        line.strip().removeprefix("- label: ").lower()
        for line in lines
        if line.strip().startswith("- label: ")
    ]
    errors: list[str] = []
    for required_label in sorted(REQUIRED_CHECK_LABELS):
        if not any(required_label in label for label in check_labels):
            errors.append(f"{relative}: missing required before-submit check `{required_label}`")
    return errors


def relative_path(root: Path, path: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


if __name__ == "__main__":
    raise SystemExit(main())
