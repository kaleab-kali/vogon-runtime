import tempfile
import unittest
from pathlib import Path

from scripts import check_dependabot_config


class CheckDependabotConfigTests(unittest.TestCase):
    def test_accepts_expected_dependabot_config(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_dependabot_config(root)

            self.assertEqual(check_dependabot_config.check_repository(root), [])

    def test_reports_missing_dependabot_config(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(
                check_dependabot_config.check_repository(Path(directory)),
                [".github/dependabot.yml: missing Dependabot configuration"],
            )

    def test_reports_missing_docker_updates(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_dependabot_config(
                root,
                dependabot_config_text().replace(docker_update_text(), ""),
            )

            self.assertEqual(
                check_dependabot_config.check_repository(root),
                [".github/dependabot.yml: missing docker updates"],
            )

    def test_reports_weakened_update_schedule(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_dependabot_config(
                root,
                dependabot_config_text().replace(
                    "package-ecosystem: cargo\n    directory: /\n    schedule:\n      interval: weekly",
                    "package-ecosystem: cargo\n    directory: /\n    schedule:\n      interval: monthly",
                ),
            )

            self.assertEqual(
                check_dependabot_config.check_repository(root),
                [".github/dependabot.yml: cargo `interval` must be 'weekly'"],
            )

    def test_reports_wrong_commit_prefix(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_dependabot_config(
                root,
                dependabot_config_text().replace(
                    "      prefix: ci",
                    "      prefix: deps",
                ),
            )

            self.assertEqual(
                check_dependabot_config.check_repository(root),
                [".github/dependabot.yml: github-actions `commit-message.prefix` must be 'ci'"],
            )

    def test_reports_missing_update_group(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_dependabot_config(
                root,
                dependabot_config_text().replace(cargo_group_text(), ""),
            )

            self.assertEqual(
                check_dependabot_config.check_repository(root),
                [
                    ".github/dependabot.yml: cargo `groups.cargo-minor-patch.patterns` must be '*'",
                    ".github/dependabot.yml: cargo `groups.cargo-minor-patch.update-types` must be 'minor,patch'",
                ],
            )


def write_dependabot_config(root: Path, text: str | None = None) -> None:
    github = root / ".github"
    github.mkdir()
    (github / "dependabot.yml").write_text(
        text or dependabot_config_text(),
        encoding="utf-8",
    )


def dependabot_config_text() -> str:
    return (
        "version: 2\n"
        "updates:\n"
        "  - package-ecosystem: cargo\n"
        "    directory: /\n"
        "    schedule:\n"
        "      interval: weekly\n"
        "    open-pull-requests-limit: 5\n"
        + cargo_group_text()
        + "    commit-message:\n"
        "      prefix: deps\n\n"
        "  - package-ecosystem: github-actions\n"
        "    directory: /\n"
        "    schedule:\n"
        "      interval: weekly\n"
        "    open-pull-requests-limit: 5\n"
        "    groups:\n"
        "      github-actions-minor-patch:\n"
        "        patterns:\n"
        "          - \"*\"\n"
        "        update-types:\n"
        "          - minor\n"
        "          - patch\n"
        "    commit-message:\n"
        "      prefix: ci\n\n"
        + docker_update_text()
    )


def cargo_group_text() -> str:
    return (
        "    groups:\n"
        "      cargo-minor-patch:\n"
        "        patterns:\n"
        "          - \"*\"\n"
        "        update-types:\n"
        "          - minor\n"
        "          - patch\n"
    )


def docker_update_text() -> str:
    return (
        "  - package-ecosystem: docker\n"
        "    directory: /\n"
        "    schedule:\n"
        "      interval: weekly\n"
        "    open-pull-requests-limit: 5\n"
        "    groups:\n"
        "      docker-minor-patch:\n"
        "        patterns:\n"
        "          - \"*\"\n"
        "        update-types:\n"
        "          - minor\n"
        "          - patch\n"
        "    commit-message:\n"
        "      prefix: deps\n"
    )


if __name__ == "__main__":
    unittest.main()
