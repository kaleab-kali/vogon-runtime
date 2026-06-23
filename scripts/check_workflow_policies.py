#!/usr/bin/env python3
"""Validate GitHub Actions workflow security policy conventions."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


WORKFLOW_SUFFIXES = {".yml", ".yaml"}
BROAD_PERMISSION_RE = re.compile(r"^\s*permissions:\s*(?:read-all|write-all)\s*$")
PERMISSION_VALUE_RE = re.compile(r"^\s+([A-Za-z0-9_-]+):\s*([A-Za-z-]+)\s*$")
TOP_LEVEL_KEY_RE = re.compile(r"^[A-Za-z0-9_-]+:")
JOB_KEY_RE = re.compile(r"^\s{2}([A-Za-z0-9_-]+):\s*$")
RUNS_ON_RE = re.compile(r"^\s{4}runs-on:\s*(.+?)\s*$")
TIMEOUT_RE = re.compile(r"^\s{4}timeout-minutes:\s*(.+?)\s*$")
ALLOWED_TOP_LEVEL_WRITE_SCOPES = {"security-events"}
FLOATING_RUNNERS = {"ubuntu-latest", "windows-latest", "macos-latest"}


@dataclass(frozen=True)
class TopLevelPermissions:
    line: int
    entries: dict[str, tuple[str, int]]


@dataclass(frozen=True)
class WorkflowJob:
    name: str
    line: int
    runs_on: str | None
    runs_on_line: int | None
    timeout_minutes: str | None
    timeout_line: int | None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="Repository root to scan. Defaults to the current directory.",
    )
    args = parser.parse_args()

    root = args.root.resolve()
    errors = check_repository(root)
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


def check_repository(root: Path) -> list[str]:
    errors: list[str] = []
    for workflow_file in workflow_files(root):
        errors.extend(check_workflow_file(root, workflow_file))
    return errors


def workflow_files(root: Path) -> list[Path]:
    workflows_dir = root / ".github" / "workflows"
    if not workflows_dir.exists():
        return []
    return sorted(
        path
        for path in workflows_dir.iterdir()
        if path.is_file() and path.suffix.lower() in WORKFLOW_SUFFIXES
    )


def check_workflow_file(root: Path, path: Path) -> list[str]:
    relative_path = path.relative_to(root).as_posix()
    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []

    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("pull_request_target:"):
            errors.append(
                f"{relative_path}:{line_number}: pull_request_target is not allowed"
            )
        if BROAD_PERMISSION_RE.match(line):
            errors.append(
                f"{relative_path}:{line_number}: broad workflow permissions are not allowed"
            )

    permissions = parse_top_level_permissions(lines)
    if permissions is None:
        errors.append(f"{relative_path}: missing top-level permissions block")
        return errors

    errors.extend(check_jobs(relative_path, lines))

    jobs_line = first_top_level_key_line(lines, "jobs:")
    if jobs_line is not None and permissions.line > jobs_line:
        errors.append(
            f"{relative_path}:{permissions.line}: top-level permissions must be before jobs"
        )

    contents = permissions.entries.get("contents")
    if contents is None:
        errors.append(
            f"{relative_path}:{permissions.line}: top-level permissions must include contents"
        )
    elif contents[0] != "read":
        errors.append(
            f"{relative_path}:{contents[1]}: top-level contents permission must be read"
        )

    for scope, (level, line_number) in permissions.entries.items():
        if level == "write" and scope not in ALLOWED_TOP_LEVEL_WRITE_SCOPES:
            errors.append(
                f"{relative_path}:{line_number}: top-level {scope} write permission "
                "must be job-scoped"
            )

    return errors


def check_jobs(relative_path: str, lines: list[str]) -> list[str]:
    jobs_line_index = next(
        (index for index, line in enumerate(lines) if line == "jobs:"),
        None,
    )
    if jobs_line_index is None:
        return []

    errors: list[str] = []
    for job in parse_jobs(lines, jobs_line_index + 1):
        if job.runs_on is None:
            errors.append(f"{relative_path}:{job.line}: job `{job.name}` missing runs-on")
        elif job.runs_on in FLOATING_RUNNERS:
            errors.append(
                f"{relative_path}:{job.runs_on_line}: job `{job.name}` uses floating runner `{job.runs_on}`"
            )

        if job.timeout_minutes is None:
            errors.append(
                f"{relative_path}:{job.line}: job `{job.name}` missing timeout-minutes"
            )
        else:
            try:
                timeout = int(job.timeout_minutes)
            except ValueError:
                errors.append(
                    f"{relative_path}:{job.timeout_line}: job `{job.name}` timeout-minutes must be an integer"
                )
            else:
                if timeout <= 0 or timeout > 60:
                    errors.append(
                        f"{relative_path}:{job.timeout_line}: job `{job.name}` timeout-minutes must be between 1 and 60"
                    )

    return errors


def parse_jobs(lines: list[str], start_index: int) -> list[WorkflowJob]:
    jobs: list[WorkflowJob] = []
    index = start_index
    while index < len(lines):
        line = lines[index]
        if is_top_level_key(line):
            break

        match = JOB_KEY_RE.match(line)
        if not match:
            index += 1
            continue

        name = match.group(1)
        line_number = index + 1
        runs_on = None
        runs_on_line = None
        timeout_minutes = None
        timeout_line = None
        index += 1
        while index < len(lines):
            child = lines[index]
            if is_top_level_key(child) or JOB_KEY_RE.match(child):
                break

            runs_on_match = RUNS_ON_RE.match(child)
            if runs_on_match:
                runs_on = runs_on_match.group(1).strip("\"'")
                runs_on_line = index + 1

            timeout_match = TIMEOUT_RE.match(child)
            if timeout_match:
                timeout_minutes = timeout_match.group(1).strip("\"'")
                timeout_line = index + 1

            index += 1

        jobs.append(
            WorkflowJob(
                name=name,
                line=line_number,
                runs_on=runs_on,
                runs_on_line=runs_on_line,
                timeout_minutes=timeout_minutes,
                timeout_line=timeout_line,
            )
        )

    return jobs


def parse_top_level_permissions(lines: list[str]) -> TopLevelPermissions | None:
    for index, line in enumerate(lines):
        if line == "permissions:":
            entries: dict[str, tuple[str, int]] = {}
            for child_index in range(index + 1, len(lines)):
                child = lines[child_index]
                if is_top_level_key(child):
                    break
                match = PERMISSION_VALUE_RE.match(child)
                if match:
                    scope, level = match.groups()
                    entries[scope] = (level, child_index + 1)
            return TopLevelPermissions(line=index + 1, entries=entries)
    return None


def first_top_level_key_line(lines: list[str], key: str) -> int | None:
    for line_number, line in enumerate(lines, start=1):
        if line == key:
            return line_number
    return None


def is_top_level_key(line: str) -> bool:
    return (
        bool(line)
        and not line.startswith((" ", "\t"))
        and bool(TOP_LEVEL_KEY_RE.match(line))
    )


if __name__ == "__main__":
    sys.exit(main())
