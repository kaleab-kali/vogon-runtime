import tempfile
import unittest
from pathlib import Path

from scripts import check_issue_templates


class CheckIssueTemplatesTests(unittest.TestCase):
    def test_accepts_valid_issue_templates(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_issue_templates(root)

            self.assertEqual(check_issue_templates.check_repository(root), [])

    def test_reports_missing_config_guards(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_issue_templates(root)
            (root / ".github" / "ISSUE_TEMPLATE" / "config.yml").write_text(
                "blank_issues_enabled: true\n",
                encoding="utf-8",
            )

            errors = check_issue_templates.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/ISSUE_TEMPLATE/config.yml: blank issues must stay disabled",
                    ".github/ISSUE_TEMPLATE/config.yml: missing private vulnerability reporting link",
                ],
            )

    def test_reports_missing_bug_reproduction_field_and_secret_check(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_issue_templates(root)
            bug_path = root / ".github" / "ISSUE_TEMPLATE" / "bug_report.yml"
            bug_path.write_text(
                valid_form(
                    name="Bug report",
                    title='title: "Bug: "',
                    label="- bug",
                    fields=["version", "component", "expected", "actual", "environment", "checks"],
                    include_secret_check=False,
                ),
                encoding="utf-8",
            )

            errors = check_issue_templates.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/ISSUE_TEMPLATE/bug_report.yml: missing required field `reproduce`",
                    ".github/ISSUE_TEMPLATE/bug_report.yml: missing required before-submit check `removed secrets`",
                ],
            )

    def test_reports_missing_feature_dropdown_option(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_issue_templates(root)
            feature_path = root / ".github" / "ISSUE_TEMPLATE" / "feature_request.yml"
            feature_path.write_text(
                valid_form(
                    name="Feature request",
                    title='title: "Feature: "',
                    label="- enhancement",
                    fields=["problem", "proposal", "area", "checks"],
                    options=["CLI", "Runtime"],
                ),
                encoding="utf-8",
            )

            errors = check_issue_templates.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/ISSUE_TEMPLATE/feature_request.yml: dropdown options missing Documentation, Other, Provider adapter, Release artifact, Replay verification",
                ],
            )

    def test_reports_stale_bug_version_placeholder(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_issue_templates(root)
            bug_path = root / ".github" / "ISSUE_TEMPLATE" / "bug_report.yml"
            bug_path.write_text(
                valid_form(
                    name="Bug report",
                    title='title: "Bug: "',
                    label="- bug",
                    fields=[
                        "version",
                        "component",
                        "expected",
                        "actual",
                        "reproduce",
                        "environment",
                        "checks",
                    ],
                    version_placeholder="vogon 0.1.0",
                ),
                encoding="utf-8",
            )

            errors = check_issue_templates.check_repository(root)

            self.assertEqual(
                errors,
                [
                    ".github/ISSUE_TEMPLATE/bug_report.yml: version placeholder must match the latest public release",
                ],
            )


def write_issue_templates(root: Path) -> None:
    template_dir = root / ".github" / "ISSUE_TEMPLATE"
    template_dir.mkdir(parents=True)
    (template_dir / "config.yml").write_text(
        "\n".join(
            [
                "blank_issues_enabled: false",
                "contact_links:",
                "  - name: Security vulnerability",
                "    url: https://github.com/kaleab-kali/vogon-runtime/security/advisories/new",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    (template_dir / "bug_report.yml").write_text(
        valid_form(
            name="Bug report",
            title='title: "Bug: "',
            label="- bug",
            fields=["version", "component", "expected", "actual", "reproduce", "environment", "checks"],
        ),
        encoding="utf-8",
    )
    (template_dir / "feature_request.yml").write_text(
        valid_form(
            name="Feature request",
            title='title: "Feature: "',
            label="- enhancement",
            fields=["problem", "proposal", "area", "checks"],
        ),
        encoding="utf-8",
    )


def valid_form(
    *,
    name: str,
    title: str,
    label: str,
    fields: list[str],
    include_secret_check: bool = True,
    options: list[str] | None = None,
    version_placeholder: str = "vogon 0.1.1",
) -> str:
    dropdown_options = options or [
        "CLI",
        "Runtime",
        "Replay verification",
        "Provider adapter",
        "Documentation",
        "Release artifact",
        "Other",
    ]
    lines = [
        f"name: {name}",
        "description: Example form.",
        title,
        "labels:",
        f"  {label}",
        "body:",
    ]

    for field in fields:
        if field in {"component", "area"}:
            lines.extend(
                [
                    "  - type: dropdown",
                    f"    id: {field}",
                    "    attributes:",
                    "      label: Area",
                    "      options:",
                    *[f"        - {option}" for option in dropdown_options],
                    "    validations:",
                    "      required: true",
                ]
            )
        elif field == "checks":
            lines.extend(
                [
                    "  - type: checkboxes",
                    "    id: checks",
                    "    attributes:",
                    "      label: Before submitting",
                    "      options:",
                ]
            )
            if include_secret_check:
                lines.append(
                    "        - label: I have removed secrets, API keys, private prompts, and sensitive replay data."
                )
                lines.append("          required: true")
            lines.extend(
                [
                    "        - label: I searched existing issues for a similar report.",
                    "          required: true",
                ]
            )
        else:
            if field == "version":
                lines.extend(
                    [
                        "  - type: input",
                        "    id: version",
                        "    attributes:",
                        "      label: version",
                        f'      placeholder: "{version_placeholder}"',
                        "    validations:",
                        "      required: true",
                    ]
                )
            else:
                lines.extend(
                    [
                        "  - type: textarea",
                        f"    id: {field}",
                        "    attributes:",
                        f"      label: {field}",
                        "    validations:",
                        "      required: true",
                    ]
                )

    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    unittest.main()
