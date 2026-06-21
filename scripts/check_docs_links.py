#!/usr/bin/env python3
"""Validate repository-local links in Markdown files."""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote, urlparse


REPO_OWNER = "kaleab-kali"
REPO_NAME = "vogon-runtime"
MARKDOWN_SUFFIXES = {".md", ".markdown"}


@dataclass(frozen=True)
class Link:
    source: Path
    line: int
    target: str


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
    for markdown_file in markdown_files(root):
        for link in extract_links(markdown_file):
            try:
                resolved = resolve_repository_link(root, markdown_file, link.target)
            except ValueError as error:
                relative_source = markdown_file.relative_to(root).as_posix()
                errors.append(f"{relative_source}:{link.line}: {error}")
                continue
            if resolved is None:
                continue
            if not resolved.exists():
                relative_source = markdown_file.relative_to(root).as_posix()
                relative_target = resolved.relative_to(root).as_posix()
                errors.append(
                    f"{relative_source}:{link.line}: missing link target "
                    f"`{link.target}` -> `{relative_target}`"
                )
    return errors


def markdown_files(root: Path) -> list[Path]:
    ignored_parts = {".git", "target"}
    return sorted(
        path
        for path in root.rglob("*")
        if path.is_file()
        and path.suffix.lower() in MARKDOWN_SUFFIXES
        and not ignored_parts.intersection(path.relative_to(root).parts)
    )


def extract_links(path: Path) -> list[Link]:
    links: list[Link] = []
    text = path.read_text(encoding="utf-8")
    for line_number, line in enumerate(text.splitlines(), start=1):
        for target in markdown_link_targets(line):
            links.append(Link(source=path, line=line_number, target=target))
    return links


def markdown_link_targets(line: str) -> list[str]:
    targets: list[str] = []
    index = 0
    while index < len(line):
        if line[index] != "[" or (index > 0 and line[index - 1] == "!"):
            index += 1
            continue

        label_end = find_matching_bracket(line, index)
        if label_end is None or label_end + 1 >= len(line) or line[label_end + 1] != "(":
            index += 1
            continue

        target_start = label_end + 2
        target_end = line.find(")", target_start)
        if target_end == -1:
            index += 1
            continue

        target = normalize_markdown_target(line[target_start:target_end])
        if target:
            targets.append(target)
        index = target_end + 1
    return targets


def find_matching_bracket(line: str, start: int) -> int | None:
    depth = 0
    for index in range(start, len(line)):
        if line[index] == "[":
            depth += 1
        elif line[index] == "]":
            depth -= 1
            if depth == 0:
                return index
    return None


def normalize_markdown_target(raw_target: str) -> str:
    target = raw_target.strip()
    if not target:
        return ""
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if " " in target:
        target = target.split(maxsplit=1)[0]
    return target


def resolve_repository_link(root: Path, source: Path, target: str) -> Path | None:
    target_without_anchor = target.split("#", maxsplit=1)[0]
    if not target_without_anchor:
        return None

    parsed = urlparse(target_without_anchor)
    if parsed.scheme:
        return resolve_github_repository_link(root, parsed)

    if target_without_anchor.startswith("/"):
        return safe_join(root, root, target_without_anchor.lstrip("/"))

    return safe_join(root, source.parent, target_without_anchor)


def resolve_github_repository_link(root: Path, parsed_url) -> Path | None:
    if parsed_url.scheme not in {"http", "https"}:
        return None
    if parsed_url.netloc.lower() != "github.com":
        return None

    parts = [unquote(part) for part in parsed_url.path.split("/") if part]
    if len(parts) < 5:
        return None
    if parts[0] != REPO_OWNER or parts[1] != REPO_NAME:
        return None
    if parts[2] not in {"blob", "tree"} or parts[3] != "main":
        return None

    return safe_join(root, root, *parts[4:])


def safe_join(root: Path, base: Path, *parts: str) -> Path:
    resolved_root = root.resolve()
    resolved = base.resolve().joinpath(*parts).resolve()
    if resolved == resolved_root or resolved_root in resolved.parents:
        return resolved
    raise ValueError(f"link target escapes repository root: {resolved}")


if __name__ == "__main__":
    sys.exit(main())
